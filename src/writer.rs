use super::Bebop;

#[derive(Debug, Clone)]
pub struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn reserve(&mut self, n: usize) {
        self.bytes.reserve(n);
    }

    pub fn write<T: Bebop>(&mut self, value: &T) {
        value.write_into(self)
    }

    pub fn write_raw(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub fn finish(self) -> Vec<u8> {
        self.bytes
    }
}
