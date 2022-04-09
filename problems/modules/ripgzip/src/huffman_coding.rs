#![forbid(unsafe_code)]

use std::{collections::HashMap, convert::TryFrom, io::BufRead, iter::zip};

use anyhow::{anyhow, Result};

use crate::bit_reader::{BitReader, BitSequence};

////////////////////////////////////////////////////////////////////////////////

pub fn decode_litlen_distance_trees<T: BufRead>(
    bit_reader: &mut BitReader<T>,
) -> Result<(HuffmanCoding<LitLenToken>, HuffmanCoding<DistanceToken>)> {
    let hlit = bit_reader.read_bits(5)?.bits();
    let hdist = bit_reader.read_bits(5)?.bits();
    let hclen = bit_reader.read_bits(4)?.bits();

    let mut tokens_lengths: Vec<(i32, u8)> = zip(
        [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ],
        vec![0u8; 19],
    )
    .collect();
    for it in tokens_lengths.iter_mut().take(hclen as usize + 4) {
        it.1 = bit_reader.read_bits(3)?.bits() as u8;
    }

    tokens_lengths.sort_by_key(|k| k.0);

    let code_lengths: Vec<u8> = tokens_lengths.into_iter().map(|(_, b)| b).collect();
    let token_tree = HuffmanCoding::<TreeCodeToken>::from_lengths(&code_lengths)?;

    let mut litlen_lengths = Vec::<u8>::new();
    let mut i = 0u16;
    while i < hlit + 257 {
        let token = token_tree.read_symbol(bit_reader)?;
        match token {
            TreeCodeToken::Length(len) => {
                litlen_lengths.push(len);
                i += 1;
            }
            TreeCodeToken::CopyPrev => {
                let count = bit_reader.read_bits(2)?.bits() + 3;
                litlen_lengths.append(&mut vec![*litlen_lengths.last().unwrap(); count as usize]);
                i += count;
            }
            TreeCodeToken::RepeatZero { base, extra_bits } => {
                let count = bit_reader.read_bits(extra_bits)?.bits() + base;
                litlen_lengths.append(&mut vec![0u8; count as usize]);
                i += count;
            }
        }
    }

    let mut dist_lengths = Vec::<u8>::new();
    i = 0;
    while i < hdist + 1 {
        let token = token_tree.read_symbol(bit_reader)?;
        match token {
            TreeCodeToken::Length(len) => {
                dist_lengths.push(len);
                i += 1;
            }
            TreeCodeToken::CopyPrev => {
                let count = bit_reader.read_bits(2)?.bits() + 3;
                dist_lengths.append(&mut vec![*dist_lengths.last().unwrap(); count as usize]);
                i += count;
            }
            TreeCodeToken::RepeatZero { base, extra_bits } => {
                let count = bit_reader.read_bits(extra_bits)?.bits() + base;
                dist_lengths.append(&mut vec![0u8; count as usize]);
                i += count;
            }
        }
    }

    Ok((
        HuffmanCoding::<LitLenToken>::from_lengths(&litlen_lengths)?,
        HuffmanCoding::<DistanceToken>::from_lengths(&dist_lengths)?,
    ))
}

pub fn default_litlen_distance_trees(
) -> Result<(HuffmanCoding<LitLenToken>, HuffmanCoding<DistanceToken>)> {
    let mut litlen_lengths = Vec::<u8>::with_capacity(288);
    litlen_lengths.append(&mut vec![8u8; 144]);
    litlen_lengths.append(&mut vec![9u8; 112]);
    litlen_lengths.append(&mut vec![7u8; 24]);
    litlen_lengths.append(&mut vec![8u8; 6]);

    let dist_lengths = vec![5u8; 32usize];

    Ok((
        HuffmanCoding::<LitLenToken>::from_lengths(&litlen_lengths)?,
        HuffmanCoding::<DistanceToken>::from_lengths(&dist_lengths)?,
    ))
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum TreeCodeToken {
    Length(u8),
    CopyPrev,
    RepeatZero { base: u16, extra_bits: u8 },
}

impl TryFrom<HuffmanCodeWord> for TreeCodeToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        match value.0 {
            value @ 0..=15 => Ok(Self::Length(value as u8)),
            16 => Ok(Self::CopyPrev),
            17 => Ok(Self::RepeatZero {
                base: 3,
                extra_bits: 3,
            }),
            18 => Ok(Self::RepeatZero {
                base: 11,
                extra_bits: 7,
            }),
            _ => Err(anyhow!("undefined tree token")),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum LitLenToken {
    Literal(u8),
    EndOfBlock,
    Length { base: u16, extra_bits: u8 },
}

impl TryFrom<HuffmanCodeWord> for LitLenToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        match value.0 {
            value @ 0..=255 => Ok(Self::Literal(value as u8)),
            256 => Ok(Self::EndOfBlock),
            value @ 257..=264 => Ok(Self::Length {
                base: value - 257 + 3,
                extra_bits: 0,
            }),
            value @ 265..=268 => Ok(Self::Length {
                base: (value - 265) * 2 + 11,
                extra_bits: 1,
            }),
            value @ 269..=272 => Ok(Self::Length {
                base: (value - 269) * 4 + 19,
                extra_bits: 2,
            }),
            value @ 273..=276 => Ok(Self::Length {
                base: (value - 273) * 8 + 35,
                extra_bits: 3,
            }),
            value @ 277..=280 => Ok(Self::Length {
                base: (value - 277) * 16 + 67,
                extra_bits: 4,
            }),
            value @ 281..=284 => Ok(Self::Length {
                base: (value - 281) * 32 + 131,
                extra_bits: 5,
            }),
            285 => Ok(Self::Length {
                base: 258,
                extra_bits: 0,
            }),
            value => Err(anyhow!("undefined litlen token {}", value)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub struct DistanceToken {
    pub base: u16,
    pub extra_bits: u8,
}

impl TryFrom<HuffmanCodeWord> for DistanceToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        match value.0 {
            0 => Ok(Self {
                extra_bits: 0,
                base: 1,
            }),
            1 => Ok(Self {
                extra_bits: 0,
                base: 2,
            }),
            2 => Ok(Self {
                extra_bits: 0,
                base: 3,
            }),
            3 => Ok(Self {
                extra_bits: 0,
                base: 4,
            }),
            4 => Ok(Self {
                extra_bits: 1,
                base: 5,
            }),
            5 => Ok(Self {
                extra_bits: 1,
                base: 7,
            }),
            6 => Ok(Self {
                extra_bits: 2,
                base: 9,
            }),
            7 => Ok(Self {
                extra_bits: 2,
                base: 13,
            }),
            8 => Ok(Self {
                extra_bits: 3,
                base: 17,
            }),
            9 => Ok(Self {
                extra_bits: 3,
                base: 25,
            }),
            10 => Ok(Self {
                extra_bits: 4,
                base: 33,
            }),
            11 => Ok(Self {
                extra_bits: 4,
                base: 49,
            }),
            12 => Ok(Self {
                extra_bits: 5,
                base: 65,
            }),
            13 => Ok(Self {
                extra_bits: 5,
                base: 97,
            }),
            14 => Ok(Self {
                extra_bits: 6,
                base: 129,
            }),
            15 => Ok(Self {
                extra_bits: 6,
                base: 193,
            }),
            16 => Ok(Self {
                extra_bits: 7,
                base: 257,
            }),
            17 => Ok(Self {
                extra_bits: 7,
                base: 385,
            }),
            18 => Ok(Self {
                extra_bits: 8,
                base: 513,
            }),
            19 => Ok(Self {
                extra_bits: 8,
                base: 769,
            }),
            20 => Ok(Self {
                extra_bits: 9,
                base: 1025,
            }),
            21 => Ok(Self {
                extra_bits: 9,
                base: 1537,
            }),
            22 => Ok(Self {
                extra_bits: 10,
                base: 2049,
            }),
            23 => Ok(Self {
                extra_bits: 10,
                base: 3073,
            }),
            24 => Ok(Self {
                extra_bits: 11,
                base: 4097,
            }),
            25 => Ok(Self {
                extra_bits: 11,
                base: 6145,
            }),
            26 => Ok(Self {
                extra_bits: 12,
                base: 8193,
            }),
            27 => Ok(Self {
                extra_bits: 12,
                base: 12289,
            }),
            28 => Ok(Self {
                extra_bits: 13,
                base: 16385,
            }),
            29 => Ok(Self {
                extra_bits: 13,
                base: 24577,
            }),
            _ => panic!(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

const MAX_BITS: usize = 15;

pub struct HuffmanCodeWord(pub u16);

pub struct HuffmanCoding<T> {
    map: HashMap<BitSequence, T>,
}

impl<T> HuffmanCoding<T>
where
    T: std::fmt::Debug + Copy + TryFrom<HuffmanCodeWord, Error = anyhow::Error>,
{
    #[allow(unused)]
    pub fn new(map: HashMap<BitSequence, T>) -> Self {
        Self { map }
    }

    #[allow(unused)]
    pub fn decode_symbol(&self, seq: BitSequence) -> Option<T> {
        self.map.get(&seq).copied()
    }

    pub fn read_symbol<U: BufRead>(&self, bit_reader: &mut BitReader<U>) -> Result<T> {
        let mut token = bit_reader.read_bits(1)?;
        loop {
            if let Some(symbol) = self.map.get(&token) {
                return Ok(*symbol);
            } else {
                token = token.concat(bit_reader.read_bits(1)?);
            }
        }
    }

    pub fn from_lengths(code_lengths: &[u8]) -> Result<Self> {
        let mut bl_count = vec![0usize; MAX_BITS + 1];
        for code_length in code_lengths {
            bl_count[*code_length as usize] += 1;
        }
        bl_count[0] = 0;

        let mut next_code = vec![0usize; MAX_BITS + 1];
        let mut code = 0usize;
        for bits in 1..=MAX_BITS {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        let mut map = HashMap::<BitSequence, T>::new();
        for (word_num, len) in code_lengths.iter().enumerate() {
            if *len != 0 {
                map.insert(
                    BitSequence::new(next_code[*len as usize] as u16, *len),
                    T::try_from(HuffmanCodeWord(word_num as u16))?,
                );
                next_code[*len as usize] += 1;
            }
        }

        Ok(Self { map })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Value(u16);

    impl TryFrom<HuffmanCodeWord> for Value {
        type Error = anyhow::Error;

        fn try_from(x: HuffmanCodeWord) -> Result<Self> {
            Ok(Self(x.0))
        }
    }

    #[test]
    fn from_lengths() -> Result<()> {
        let code = HuffmanCoding::<Value>::from_lengths(&[2, 3, 4, 3, 3, 4, 2])?;

        assert_eq!(
            code.decode_symbol(BitSequence::new(0b00, 2)),
            Some(Value(0)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b100, 3)),
            Some(Value(1)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b1110, 4)),
            Some(Value(2)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b101, 3)),
            Some(Value(3)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b110, 3)),
            Some(Value(4)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b1111, 4)),
            Some(Value(5)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b01, 2)),
            Some(Value(6)),
        );

        assert_eq!(code.decode_symbol(BitSequence::new(0b0, 1)), None);
        assert_eq!(code.decode_symbol(BitSequence::new(0b10, 2)), None);
        assert_eq!(code.decode_symbol(BitSequence::new(0b111, 3)), None,);

        Ok(())
    }

    #[test]
    fn read_symbol() -> Result<()> {
        let code = HuffmanCoding::<Value>::from_lengths(&[2, 3, 4, 3, 3, 4, 2])?;
        let mut data: &[u8] = &[0b10111001, 0b11001010, 0b11101101];
        let mut reader = BitReader::new(&mut data);

        assert_eq!(code.read_symbol(&mut reader)?, Value(1));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(3));
        assert_eq!(code.read_symbol(&mut reader)?, Value(6));
        assert_eq!(code.read_symbol(&mut reader)?, Value(0));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(4));
        assert!(code.read_symbol(&mut reader).is_err());

        Ok(())
    }
}
