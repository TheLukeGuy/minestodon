use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::num::TryFromIntError;
use std::ops::{BitAnd, BitOrAssign, Shl};
use uuid::Uuid;

pub trait PacketReadExt: ReadBytesExt {
    fn read_bool(&mut self) -> Result<bool> {
        let byte = self.read_u8().context("failed to read the boolean byte")?;
        Ok(byte != 0)
    }

    fn read_var<V: VarInt>(&mut self) -> Result<V> {
        let mut partial = PartialVarInt::new();
        loop {
            let next = self.read_u8().context("failed to read the next byte")?;
            match partial.next(next)? {
                PartialVarInt::Full(full) => return Ok(full),
                continued => partial = continued,
            }
        }
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self
            .read_var::<i32>()
            .context("failed to read the string length")?
            .try_into()
            .context("the string length doesn't fit in a usize")?;

        let mut bytes = vec![0; len];
        for byte in bytes.iter_mut() {
            *byte = self.read_u8().context("failed to read the next byte")?;
        }
        String::from_utf8(bytes).context("the string is not valid UTF-8")
    }

    fn read_uuid(&mut self) -> Result<Uuid> {
        let high = self
            .read_u64::<BigEndian>()
            .context("failed to read the high bits")?;
        let low = self
            .read_u64::<BigEndian>()
            .context("failed to read the low bits")?;
        Ok(Uuid::from_u64_pair(high, low))
    }
}

impl<R: ReadBytesExt> PacketReadExt for R {}

pub trait PacketWriteExt: WriteBytesExt {
    fn write_bool(&mut self, bool: bool) -> Result<()> {
        let byte = if bool { 1 } else { 0 };
        self.write_u8(byte)
            .context("failed to write the boolean byte")
    }

    fn write_var<V: VarInt>(&mut self, mut var: V) -> Result<()> {
        let zero = V::from(0);
        let segment_bits = V::from(0x7f);
        loop {
            let next = (var & segment_bits).try_into().unwrap();
            var = var.unsigned_shr(7);
            if var == zero {
                return self.write_u8(next).context("failed to write the last byte");
            } else {
                let next = next | 0x80;
                self.write_u8(next)
                    .context("failed to write the next byte")?;
            }
        }
    }

    fn write_str(&mut self, str: &str) -> Result<()> {
        let len = str
            .len()
            .try_into()
            .context("the string length doesn't fit in an i32")?;

        self.write_var::<i32>(len)
            .context("failed to write the string length")?;
        for byte in str.bytes() {
            self.write_u8(byte)
                .context("failed to write the next byte")?;
        }
        Ok(())
    }

    fn write_uuid(&mut self, uuid: &Uuid) -> Result<()> {
        let (high, low) = uuid.as_u64_pair();
        self.write_u64::<BigEndian>(high)
            .context("failed to write the high bits")?;
        self.write_u64::<BigEndian>(low)
            .context("failed to write the low bits")
    }
}

impl<W: WriteBytesExt> PacketWriteExt for W {}

pub trait VarInt:
    Copy
    + PartialEq
    + From<u8>
    + TryInto<u8, Error = TryFromIntError>
    + BitAnd<Output = Self>
    + BitOrAssign<Self>
    + Shl<usize, Output = Self>
{
    const MAX_VAR_LEN: usize;

    fn unsigned_shr(self, rhs: usize) -> Self;
}

impl VarInt for i32 {
    const MAX_VAR_LEN: usize = 5;

    fn unsigned_shr(self, rhs: usize) -> Self {
        (self as u32 >> rhs) as Self
    }
}

impl VarInt for i64 {
    const MAX_VAR_LEN: usize = 10;

    fn unsigned_shr(self, rhs: usize) -> Self {
        (self as u64 >> rhs) as Self
    }
}

pub enum PartialVarInt<V: VarInt> {
    Partial(Vec<u8>),
    Full(V),
}

impl<V: VarInt> PartialVarInt<V> {
    pub fn new() -> Self {
        Self::Partial(vec![])
    }

    pub fn next(self, byte: u8) -> Result<Self> {
        let Self::Partial(mut bytes) = self else {
            return Ok(self);
        };
        bytes.push(byte);

        let last = byte >> 7 == 0;
        if !last {
            if bytes.len() == V::MAX_VAR_LEN {
                bail!(
                    "the VarInt is too long; it should be no longer than {} bytes",
                    V::MAX_VAR_LEN
                );
            }
            return Ok(Self::Partial(bytes));
        }

        let mut full = V::from(0);
        for (group, byte) in bytes.into_iter().enumerate() {
            let part = V::from(byte & 0x7f) << (group * 7);
            full |= part;
        }
        Ok(Self::Full(full))
    }
}

impl<V: VarInt> Default for PartialVarInt<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    #[test]
    fn read_zero_var_i32() -> Result<()> {
        read_var::<i32>(0, &[0])
    }

    #[test]
    fn write_zero_var_i32() -> Result<()> {
        write_var::<i32>(0, &[0])
    }

    #[test]
    fn read_simple_var_i32() -> Result<()> {
        read_var::<i32>(2, &[2])
    }

    #[test]
    fn write_simple_var_i32() -> Result<()> {
        write_var::<i32>(2, &[2])
    }

    #[test]
    fn read_min_var_i32() -> Result<()> {
        read_var(i32::MIN, &[0x80, 0x80, 0x80, 0x80, 0x08])
    }

    #[test]
    fn write_min_var_i32() -> Result<()> {
        write_var(i32::MIN, &[0x80, 0x80, 0x80, 0x80, 0x08])
    }

    #[test]
    fn read_max_var_i32() -> Result<()> {
        read_var(i32::MAX, &[0xff, 0xff, 0xff, 0xff, 0x07])
    }

    #[test]
    fn write_max_var_i32() -> Result<()> {
        write_var(i32::MAX, &[0xff, 0xff, 0xff, 0xff, 0x07])
    }

    #[test]
    fn read_zero_var_i64() -> Result<()> {
        read_var::<i64>(0, &[0])
    }

    #[test]
    fn write_zero_var_i64() -> Result<()> {
        write_var::<i64>(0, &[0])
    }

    #[test]
    fn read_simple_var_i64() -> Result<()> {
        read_var::<i64>(2, &[2])
    }

    #[test]
    fn write_simple_var_i64() -> Result<()> {
        write_var::<i64>(2, &[2])
    }

    #[test]
    fn read_min_var_i64() -> Result<()> {
        read_var(
            i64::MIN,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        )
    }

    #[test]
    fn write_min_var_i64() -> Result<()> {
        write_var(
            i64::MIN,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        )
    }

    #[test]
    fn read_max_var_i64() -> Result<()> {
        read_var(
            i64::MAX,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        )
    }

    #[test]
    fn write_max_var_i64() -> Result<()> {
        write_var(
            i64::MAX,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        )
    }

    fn read_var<V: VarInt + Debug>(expected: V, mut bytes: &[u8]) -> Result<()> {
        assert_eq!(expected, bytes.read_var()?);
        Ok(())
    }

    fn write_var<V: VarInt>(var: V, expected: &[u8]) -> Result<()> {
        let mut buf = vec![];
        buf.write_var(var)?;
        assert_eq!(expected, buf);
        Ok(())
    }

    const TEST_STRING: &str = "Hello Minestodon";

    #[test]
    fn read_string() -> Result<()> {
        let buf = test_string_bytes()?;
        let string = (&buf[..]).read_string()?;
        assert_eq!(TEST_STRING, string);
        Ok(())
    }

    #[test]
    fn write_str() -> Result<()> {
        let mut buf = vec![];
        buf.write_str(TEST_STRING)?;
        assert_eq!(test_string_bytes()?, buf);
        Ok(())
    }

    fn test_string_bytes() -> Result<Vec<u8>> {
        let mut buf = vec![];
        buf.write_var::<i32>(TEST_STRING.len().try_into()?)?;
        buf.extend_from_slice(TEST_STRING.as_bytes());
        Ok(buf)
    }
}
