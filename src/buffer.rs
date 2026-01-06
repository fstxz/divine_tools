/// Helper struct for reading from a buffer of bytes.
pub struct BufferReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BufferReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.position == self.buffer.len()
    }

    pub fn skip(&mut self, count: usize) {
        self.position += count;
    }

    pub fn read_u8(&mut self) -> crate::Result<u8> {
        let bytes = self.read_bytes(1)?;
        Ok(u8::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_bool(&mut self) -> crate::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_i16(&mut self) -> crate::Result<i16> {
        let bytes = self.read_bytes(2)?;
        Ok(i16::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_u32(&mut self) -> crate::Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_i32(&mut self) -> crate::Result<i32> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_f32(&mut self) -> crate::Result<f32> {
        let bytes = self.read_bytes(4)?;
        Ok(f32::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_string(&mut self) -> crate::Result<String> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_bytes(length)?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    pub fn read_bytes(&mut self, count: usize) -> crate::Result<&[u8]> {
        if self.position + count > self.buffer.len() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Not enough bytes in buffer (position {})", self.position),
            )));
        }

        let slice = &self.buffer[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }
}

/// Helper struct for writing data to a buffer.
pub struct BufferWriter {
    buffer: Vec<u8>,
}

impl BufferWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.buffer.push(value);
        self
    }

    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.write_u8(value as u8);
        self
    }

    pub fn write_i16(&mut self, value: i16) -> &mut Self {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn write_i32(&mut self, value: i32) -> &mut Self {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn write_f32(&mut self, value: f32) -> &mut Self {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn write_string(&mut self, value: &str) -> &mut Self {
        self.write_u32(value.len() as u32);
        self.write_bytes(value.as_bytes());
        self
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.buffer.extend_from_slice(bytes);
        self
    }

    pub fn pad(&mut self, count: usize) -> &mut Self {
        for _ in 0..count {
            self.buffer.push(0);
        }
        self
    }

    pub fn finish(self) -> Vec<u8> {
        self.buffer.to_vec()
    }
}
