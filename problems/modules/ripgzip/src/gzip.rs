#![forbid(unsafe_code)]

use std::io::{BufRead, Write};

use anyhow::{anyhow, bail, Result};
use crc::Crc;

use crate::{bit_reader::BitReader, deflate::DeflateReader, tracking_writer::TrackingWriter};

////////////////////////////////////////////////////////////////////////////////

const ID1: u8 = 0x1f;
const ID2: u8 = 0x8b;

const CM_DEFLATE: u8 = 8;

const FTEXT_OFFSET: u8 = 0;
const FHCRC_OFFSET: u8 = 1;
const FEXTRA_OFFSET: u8 = 2;
const FNAME_OFFSET: u8 = 3;
const FCOMMENT_OFFSET: u8 = 4;

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MemberHeader {
    pub compression_method: CompressionMethod,
    pub modification_time: u32,
    pub extra: Option<Vec<u8>>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub extra_flags: u8,
    pub os: u8,
    pub has_crc: bool,
    pub is_text: bool,
}

impl MemberHeader {
    pub fn crc16(&self) -> u16 {
        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        digest.update(&[ID1, ID2, self.compression_method.into(), self.flags().0]);
        digest.update(&self.modification_time.to_le_bytes());
        digest.update(&[self.extra_flags, self.os]);

        if let Some(extra) = &self.extra {
            digest.update(&(extra.len() as u16).to_le_bytes());
            digest.update(extra);
        }

        if let Some(name) = &self.name {
            digest.update(name.as_bytes());
            digest.update(&[0]);
        }

        if let Some(comment) = &self.comment {
            digest.update(comment.as_bytes());
            digest.update(&[0]);
        }

        (digest.finalize() & 0xffff) as u16
    }

    pub fn flags(&self) -> MemberFlags {
        let mut flags = MemberFlags(0);
        flags.set_is_text(self.is_text);
        flags.set_has_crc(self.has_crc);
        flags.set_has_extra(self.extra.is_some());
        flags.set_has_name(self.name.is_some());
        flags.set_has_comment(self.comment.is_some());
        flags
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompressionMethod {
    Deflate,
    Unknown(u8),
}

impl From<u8> for CompressionMethod {
    fn from(value: u8) -> Self {
        match value {
            CM_DEFLATE => Self::Deflate,
            x => Self::Unknown(x),
        }
    }
}

impl From<CompressionMethod> for u8 {
    fn from(method: CompressionMethod) -> u8 {
        match method {
            CompressionMethod::Deflate => CM_DEFLATE,
            CompressionMethod::Unknown(x) => x,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MemberFlags(u8);

#[allow(unused)]
impl MemberFlags {
    fn bit(&self, n: u8) -> bool {
        (self.0 >> n) & 1 != 0
    }

    fn set_bit(&mut self, n: u8, value: bool) {
        if value {
            self.0 |= 1 << n;
        } else {
            self.0 &= !(1 << n);
        }
    }

    pub fn is_text(&self) -> bool {
        self.bit(FTEXT_OFFSET)
    }

    pub fn set_is_text(&mut self, value: bool) {
        self.set_bit(FTEXT_OFFSET, value)
    }

    pub fn has_crc(&self) -> bool {
        self.bit(FHCRC_OFFSET)
    }

    pub fn set_has_crc(&mut self, value: bool) {
        self.set_bit(FHCRC_OFFSET, value)
    }

    pub fn has_extra(&self) -> bool {
        self.bit(FEXTRA_OFFSET)
    }

    pub fn set_has_extra(&mut self, value: bool) {
        self.set_bit(FEXTRA_OFFSET, value)
    }

    pub fn has_name(&self) -> bool {
        self.bit(FNAME_OFFSET)
    }

    pub fn set_has_name(&mut self, value: bool) {
        self.set_bit(FNAME_OFFSET, value)
    }

    pub fn has_comment(&self) -> bool {
        self.bit(FCOMMENT_OFFSET)
    }

    pub fn set_has_comment(&mut self, value: bool) {
        self.set_bit(FCOMMENT_OFFSET, value)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MemberFooter {
    pub data_crc32: u32,
    pub data_size: u32,
}

////////////////////////////////////////////////////////////////////////////////

pub struct GzipReader<T, W> {
    reader: T,
    writer: W,
}

impl<T: BufRead, W: Write> GzipReader<T, W> {
    pub fn new(reader: T, writer: W) -> Self {
        Self { reader, writer }
    }

    pub fn decode_data(&mut self) -> Result<()> {
        let mut member_reader = MemberReader {
            inner: &mut self.reader,
        };
        member_reader.decode_data(&mut self.writer)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct MemberReader<T: BufRead> {
    inner: T,
}

impl<T: BufRead> MemberReader<T> {
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn decode_data<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        loop {
            let member_header = match self.read_header() {
                Ok((member_header, _)) => member_header,
                anyhow::Result::Err(err) => {
                    let anyhow_err = anyhow!(err);
                    if anyhow_err.to_string().contains("done") {
                        return Ok(());
                    } else {
                        return Err(anyhow_err);
                    }
                }
            };

            if member_header.compression_method != CompressionMethod::Deflate {
                bail!("unsupported compression method")
            }

            let tracking_writer = TrackingWriter::new(&mut *writer);
            let deflate_reader =
                DeflateReader::new(BitReader::new(self.inner_mut()), tracking_writer);

            let (size, crc32) = deflate_reader.decode_data()?;

            let footer = self.read_footer()?;

            if footer.data_size != size {
                bail!("length check failed")
            }

            if footer.data_crc32 != crc32 {
                bail!("crc32 check failed")
            }
        }
    }

    pub fn read_header(&mut self) -> Result<(MemberHeader, MemberFlags)> {
        let mut bit_reader = BitReader::new(self.inner_mut());

        let try_read = bit_reader.read_bits(8);
        if try_read.is_err() {
            bail!("done");
        }

        let id1 = try_read.unwrap().bits() as u8;
        if id1 != ID1 {
            bail!("wrong id values");
        }

        let id2 = bit_reader.read_bits(8)?.bits() as u8;
        if id2 != ID2 {
            bail!("wrong id values");
        }

        let compression_method = CompressionMethod::from(bit_reader.read_bits(8)?.bits() as u8);
        let member_flags = MemberFlags(bit_reader.read_bits(8)?.bits() as u8);

        let mtime_word1 = bit_reader.read_bits(16)?.bits() as u32;
        let mtime_word2 = bit_reader.read_bits(16)?.bits() as u32;

        let modification_time = (mtime_word2 << 16) | mtime_word1;

        let extra_flags = bit_reader.read_bits(8)?.bits() as u8;
        let os = bit_reader.read_bits(8)?.bits() as u8;

        let extra = if member_flags.has_extra() {
            let xlen = bit_reader.read_bits(16)?.bits() as usize;
            let mut extra = vec![0u8; xlen];

            for it in extra.iter_mut().take(xlen) {
                *it = bit_reader.read_bits(8)?.bits() as u8;
            }

            Some(extra)
        } else {
            None
        };

        let name = if member_flags.has_name() {
            Some(MemberReader::<T>::read_null_terminated_str(
                &mut bit_reader,
            )?)
        } else {
            None
        };

        let comment = if member_flags.has_comment() {
            Some(MemberReader::<T>::read_null_terminated_str(
                &mut bit_reader,
            )?)
        } else {
            None
        };

        let member_header = MemberHeader {
            compression_method,
            modification_time,
            extra,
            name,
            comment,
            extra_flags,
            os,
            has_crc: member_flags.has_crc(),
            is_text: member_flags.is_text(),
        };

        if member_flags.has_crc() && member_header.crc16() != bit_reader.read_bits(16)?.bits() {
            bail!("header crc16 check failed");
        }

        Ok((member_header, member_flags))
    }

    pub fn read_null_terminated_str(reader: &mut BitReader<&mut T>) -> Result<String> {
        let mut str_bytes = Vec::<u8>::new();
        loop {
            let byte = reader.read_bits(8)?.bits() as u8;
            if byte == 0 {
                break;
            }

            str_bytes.push(byte);
        }

        let str = String::from_utf8(str_bytes)?;

        Ok(str)
    }

    pub fn read_footer(&mut self) -> Result<MemberFooter> {
        let mut bit_reader = BitReader::<&mut T>::new(self.inner_mut());

        let crc32_word1 = bit_reader.read_bits(16)?.bits() as u32;
        let crc32_word2 = bit_reader.read_bits(16)?.bits() as u32;

        let data_crc32 = (crc32_word2 << 16) | crc32_word1;

        let isize_word1 = bit_reader.read_bits(16)?.bits() as u32;
        let isize_word2 = bit_reader.read_bits(16)?.bits() as u32;

        let data_size = (isize_word2 << 16) | isize_word1;

        Ok(MemberFooter {
            data_crc32,
            data_size,
        })
    }
}
