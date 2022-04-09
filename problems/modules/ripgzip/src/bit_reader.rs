#![forbid(unsafe_code)]

use std::io::{self, BufRead};

use byteorder::{LittleEndian, ReadBytesExt};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitSequence {
    bits: u16,
    len: u8,
}

impl BitSequence {
    pub fn new(bits: u16, len: u8) -> Self {
        let shift = u16::BITS - len as u32;
        Self {
            bits: (bits.checked_shl(shift).unwrap_or(0))
                .checked_shr(shift)
                .unwrap_or(0),
            len,
        }
    }

    pub fn bits(&self) -> u16 {
        self.bits
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn concat(self, other: Self) -> Self {
        Self {
            bits: (self.bits() << other.len()) | other.bits(),
            len: self.len() + other.len(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct BitReader<T> {
    stream: T,
    last_bits: u16,
    last_len: u8,
}

impl<T: BufRead> BitReader<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            last_bits: 0,
            last_len: 0,
        }
    }

    pub fn read_bits(&mut self, mut len: u8) -> io::Result<BitSequence> {
        if self.last_len >= len {
            self.last_len -= len;

            let bits = self.last_bits;

            self.last_bits >>= len;

            Ok(BitSequence::new(bits, len))
        } else {
            let buf_bits = BitSequence::new(self.last_bits, self.last_len);

            len -= self.last_len;

            let bits = if len as u32 > u8::BITS {
                self.last_len = u16::BITS as u8 - len;
                self.stream.read_u16::<LittleEndian>()?
            } else {
                self.last_len = u8::BITS as u8 - len;
                self.stream.read_u8()? as u16
            };

            if self.last_len > 0 {
                self.last_bits = bits >> len;
            }

            Ok(BitSequence::new(bits, len).concat(buf_bits))
        }
    }

    /// Discard all the unread bits in the current byte and return a mutable reference
    /// to the underlying reader.
    pub fn borrow_reader_from_boundary(&mut self) -> &mut T {
        self.last_len = 0;
        &mut self.stream
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::ReadBytesExt;

    #[test]
    fn read_bits() -> io::Result<()> {
        let data: &[u8] = &[0b01100011, 0b11011011, 0b10101111];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_bits(1)?, BitSequence::new(0b1, 1));
        assert_eq!(reader.read_bits(2)?, BitSequence::new(0b01, 2));
        assert_eq!(reader.read_bits(3)?, BitSequence::new(0b100, 3));
        assert_eq!(reader.read_bits(4)?, BitSequence::new(0b1101, 4));
        assert_eq!(reader.read_bits(5)?, BitSequence::new(0b10110, 5));
        assert_eq!(reader.read_bits(8)?, BitSequence::new(0b01011111, 8));
        assert_eq!(
            reader.read_bits(2).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );

        Ok(())
    }

    #[test]
    fn borrow_reader_from_boundary() -> io::Result<()> {
        let data: &[u8] = &[0b01100011, 0b11011011, 0b10101111];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_bits(3)?, BitSequence::new(0b011, 3));
        assert_eq!(reader.borrow_reader_from_boundary().read_u8()?, 0b11011011);
        assert_eq!(reader.read_bits(8)?, BitSequence::new(0b10101111, 8));
        Ok(())
    }
}
