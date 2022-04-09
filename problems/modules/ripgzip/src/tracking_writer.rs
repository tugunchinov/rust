#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::io::{self, Write};

use anyhow::{anyhow, Result};
use crc::{Crc, Digest, CRC_32_ISO_HDLC};

////////////////////////////////////////////////////////////////////////////////

const HISTORY_SIZE: usize = 32768;
const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub struct TrackingWriter<T: Write> {
    inner: T,
    buf: VecDeque<u8>,
    bytes: usize,
    digest: Digest<'static, u32>,
}

impl<T: Write> Write for TrackingWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes = self.inner.write(buf)?;
        self.bytes += bytes;
        self.digest.update(&buf[0..bytes]);

        for byte in &buf[0..bytes] {
            if self.buf.len() == HISTORY_SIZE {
                self.buf.pop_front();
            }
            self.buf.push_back(*byte);
        }

        Ok(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Write> TrackingWriter<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            buf: VecDeque::with_capacity(HISTORY_SIZE),
            bytes: 0usize,
            digest: CRC.digest(),
        }
    }

    /// Write a sequence of `len` bytes written `dist` bytes ago.
    pub fn write_previous(&mut self, dist: usize, len: usize) -> Result<()> {
        if dist > self.buf.len() {
            Err(anyhow!(
                "dist {dist} is greater than buffer length {}",
                self.buf.len()
            ))
        } else {
            let start = self.buf.len() - dist;
            let buf: Vec<u8> = self.buf.make_contiguous()[start..]
                .iter()
                .cycle()
                .take(len)
                .cloned()
                .collect();
            Ok(self.write_all(&buf)?)
        }
    }

    pub fn byte_count(&self) -> usize {
        self.bytes
    }

    pub fn crc32(self) -> u32 {
        let digest = self.digest;
        digest.finalize()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;

    #[test]
    fn write() -> Result<()> {
        let mut buf: &mut [u8] = &mut [0u8; 10];
        let mut writer = TrackingWriter::new(&mut buf);

        assert_eq!(writer.write(&[1, 2, 3, 4])?, 4);
        assert_eq!(writer.byte_count(), 4);

        assert_eq!(writer.write(&[4, 8, 15, 16, 23])?, 5);
        assert_eq!(writer.byte_count(), 9);

        assert_eq!(writer.write(&[0, 0, 123])?, 1);
        assert_eq!(writer.byte_count(), 10);

        assert_eq!(writer.write(&[42, 124, 234, 27])?, 0);
        assert_eq!(writer.byte_count(), 10);
        assert_eq!(writer.crc32(), 2992191065);

        Ok(())
    }

    #[test]
    fn write_previous() -> Result<()> {
        let mut buf: &mut [u8] = &mut [0u8; 512];
        let mut writer = TrackingWriter::new(&mut buf);

        for i in 0..=255 {
            writer.write_u8(i)?;
        }

        writer.write_previous(192, 128)?;
        assert_eq!(writer.byte_count(), 384);

        assert!(writer.write_previous(10000, 20).is_err());
        assert_eq!(writer.byte_count(), 384);

        assert!(writer.write_previous(256, 256).is_err());
        assert_eq!(writer.byte_count(), 512);

        assert!(writer.write_previous(1, 1).is_err());
        assert_eq!(writer.byte_count(), 512);
        assert_eq!(writer.crc32(), 2733545866);

        Ok(())
    }
}
