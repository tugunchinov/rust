#![forbid(unsafe_code)]

use std::io::{BufRead, Write};

use anyhow::Result;

use crate::gzip::GzipReader;

mod bit_reader;
mod deflate;
mod gzip;
mod huffman_coding;
mod tracking_writer;

pub fn decompress<R: BufRead, W: Write>(input: R, output: W) -> Result<()> {
    let mut reader = GzipReader::new(input, output);

    reader.decode_data()?;

    Ok(())
}
