#![forbid(unsafe_code)]

use std::io::{BufRead, Write};

use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::huffman_coding::LitLenToken;
use crate::tracking_writer::TrackingWriter;
use crate::{
    bit_reader::BitReader,
    huffman_coding::{
        decode_litlen_distance_trees, default_litlen_distance_trees, DistanceToken, HuffmanCoding,
    },
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BlockHeader {
    pub is_final: bool,
    pub compression_type: CompressionType,
}

#[derive(Debug, PartialEq)]
pub enum CompressionType {
    Uncompressed = 0,
    FixedTree = 1,
    DynamicTree = 2,
    Reserved = 3,
}

////////////////////////////////////////////////////////////////////////////////

pub struct DeflateReader<T, W: Write> {
    bit_reader: BitReader<T>,
    writer: TrackingWriter<W>,
}

impl<T: BufRead, W: Write> DeflateReader<T, W> {
    pub fn new(bit_reader: BitReader<T>, writer: TrackingWriter<W>) -> Self {
        Self { bit_reader, writer }
    }

    pub fn decode_data(mut self) -> Result<(u32, u32)> {
        loop {
            let header = self.next_block()?;

            match header.compression_type {
                CompressionType::Uncompressed => {
                    let reader = self.bit_reader.borrow_reader_from_boundary();

                    let len = reader.read_u16::<LittleEndian>()?;
                    let nlen = reader.read_u16::<LittleEndian>()?;

                    if len != !nlen {
                        return Err(anyhow!("nlen check failed"));
                    }

                    let mut block_data = vec![0u8; len as usize];
                    reader.read_exact(&mut block_data)?;

                    self.writer.write_all(&block_data)?;
                }

                CompressionType::FixedTree => {
                    let (litlen_tree, distance_tree) = default_litlen_distance_trees()?;
                    self.read_block(litlen_tree, distance_tree)?;
                }

                CompressionType::DynamicTree => {
                    let (litlen_tree, distance_tree) =
                        decode_litlen_distance_trees(&mut self.bit_reader)?;
                    self.read_block(litlen_tree, distance_tree)?;
                }

                CompressionType::Reserved => return Err(anyhow!("unsupported block type")),
            }

            if header.is_final {
                break;
            }
        }

        Ok((self.writer.byte_count() as u32, self.writer.crc32()))
    }

    pub fn read_block(
        &mut self,
        litlen_tree: HuffmanCoding<LitLenToken>,
        distance_tree: HuffmanCoding<DistanceToken>,
    ) -> Result<()> {
        loop {
            let litlen_value = litlen_tree.read_symbol(&mut self.bit_reader)?;

            match litlen_value {
                LitLenToken::Literal(literal) => {
                    self.writer.write_u8(literal)?;
                }
                LitLenToken::EndOfBlock => {
                    break;
                }
                LitLenToken::Length { base, extra_bits } => {
                    let len = base + self.bit_reader.read_bits(extra_bits)?.bits();

                    let dist_token = distance_tree.read_symbol(&mut self.bit_reader)?;
                    let dist =
                        dist_token.base + self.bit_reader.read_bits(dist_token.extra_bits)?.bits();

                    self.writer.write_previous(dist as usize, len as usize)?;
                }
            }
        }

        Ok(())
    }

    pub fn next_block(&mut self) -> Result<BlockHeader> {
        let is_final = match self.bit_reader.read_bits(1) {
            Ok(bit_seq) => bit_seq.bits() == 1,
            Err(err) => return Err(anyhow!(err)),
        };

        let btype = match self.bit_reader.read_bits(2) {
            Ok(bit_seq) => bit_seq.bits(),
            Err(err) => return Err(anyhow!(err)),
        };

        let compression_type = match btype {
            0 => CompressionType::Uncompressed,
            1 => CompressionType::FixedTree,
            2 => CompressionType::DynamicTree,
            3 => CompressionType::Reserved,
            _ => return Err(anyhow!("undefined type")),
        };

        Ok(BlockHeader {
            is_final,
            compression_type,
        })
    }
}
