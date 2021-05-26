mod date;
mod error;
mod guid;
mod reader;
mod writer;

pub use date::Date;
pub use error::Error;
pub use guid::Guid;
pub use reader::Reader;
pub use writer::Writer;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;

#[doc(hidden)]
pub mod __export {
    pub use bebop_macro::read_bebop;
}

#[macro_export]
macro_rules! bebop {
    ($($path:literal),* $(,)?) => (
        $($crate::__export::read_bebop!($crate $path);)*
    );
}

pub trait Bebop: Sized {
    fn read_from(reader: &mut Reader) -> Result<Self, Error>;
    fn write_into(&self, writer: &mut Writer);

    fn decode(bytes: &[u8]) -> Result<Self, Error> {
        Reader::new(bytes).read()
    }

    fn encode(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        writer.write(self);
        writer.finish()
    }
}

pub trait Opcode {
    const OPCODE: u32;
}

impl Bebop for bool {
    fn read_from(reader: &mut Reader) -> Result<Self, Error> {
        Ok(reader.read_raw(1)?[0] != 0)
    }
    fn write_into(&self, writer: &mut Writer) {
        writer.write_raw(&[*self as u8])
    }
}

macro_rules! impl_int {
    ($($t:ty)*) => ($(
        impl Bebop for $t {
            fn read_from(reader: &mut Reader) -> Result<Self, Error> {
                Ok(<$t>::from_le_bytes(*reader.read_raw_array()?))
            }
            fn write_into(&self, writer: &mut Writer) {
                writer.write_raw(&self.to_le_bytes())
            }
        }
    )*);
}

impl_int!(u8 i8 u16 i16 u32 i32 u64 i64 usize isize f32 f64);

impl Bebop for String {
    fn read_from(reader: &mut Reader) -> Result<Self, Error> {
        let len: u32 = reader.read()?;
        let bytes = reader.read_raw(len as usize)?;
        Ok(String::from(std::str::from_utf8(bytes)?))
    }
    fn write_into(&self, writer: &mut Writer) {
        writer.write(&u32::try_from(self.len()).unwrap());
        writer.write_raw(self.as_bytes());
    }
}

impl Bebop for Guid {
    fn read_from(reader: &mut Reader) -> Result<Self, Error> {
        Ok(Guid::from(*reader.read_raw_array()?))
    }
    fn write_into(&self, writer: &mut Writer) {
        writer.write_raw(&<[u8; 16]>::from(*self))
    }
}

impl Bebop for Date {
    fn read_from(reader: &mut Reader) -> Result<Self, Error> {
        Ok(Date(reader.read()?))
    }
    fn write_into(&self, writer: &mut Writer) {
        writer.write(&self.0)
    }
}

impl<T: Bebop> Bebop for Vec<T> {
    fn read_from(reader: &mut Reader) -> Result<Self, Error> {
        let len: u32 = reader.read()?;
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(reader.read()?);
        }
        Ok(vec)
    }
    fn write_into(&self, writer: &mut Writer) {
        writer.write(&u32::try_from(self.len()).unwrap());
        for item in self {
            writer.write(item);
        }
    }
}

impl<K: Bebop + Hash + Eq, V: Bebop> Bebop for HashMap<K, V> {
    fn read_from(reader: &mut Reader) -> Result<Self, Error> {
        let len: u32 = reader.read()?;
        let mut map = HashMap::with_capacity(len as usize);
        for _ in 0..len {
            map.insert(reader.read()?, reader.read()?);
        }
        Ok(map)
    }
    fn write_into(&self, writer: &mut Writer) {
        writer.write(&u32::try_from(self.len()).unwrap());
        for (key, value) in self {
            writer.write(key);
            writer.write(value);
        }
    }
}
// bytes (Vec<u8>?)
// Vec<T>
// Map<K, V>
//impl<T: Bebop> Bebop for Vec<T> {
//    fn read_from(read: &mut impl Read) -> Result<Self>;
//    fn encode(&self, write: &mut impl Write) -> Result<()>;
//}

#[cfg(test)]
mod test {
    use crate::Bebop;

    #[test]
    fn string() {
        let s = String::from("hello world");
        let msg = s.encode();
        assert_eq!(msg, b"\x0b\0\0\0hello world");
        let s2 = String::decode(&msg).unwrap();
        assert_eq!(s, s2);
    }
}
