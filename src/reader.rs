use super::Bebop;
use super::Error;
use std::convert::TryInto;

#[derive(Debug, Clone)]
pub struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn read<T: Bebop>(&mut self) -> Result<T, Error> {
        T::read_from(self)
    }

    pub fn read_raw(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.bytes.len() < n {
            return Err(Error::UnexpectedEnd);
        }
        let (read, rest) = self.bytes.split_at(n);
        self.bytes = rest;
        Ok(read)
    }

    pub fn read_raw_array<const N: usize>(&mut self) -> Result<&'a [u8; N], Error> {
        Ok(self.read_raw(N)?.try_into().unwrap())
    }

    pub fn bytes_left(&self) -> &'a [u8] {
        self.bytes
    }
}
