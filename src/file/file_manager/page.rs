#[derive(Debug, Clone)]
pub struct Page {
    pub buffer: Vec<u8>,
}

impl Page {
    pub fn new(block_size: u64) -> Self {
        Page {
            buffer: vec![0; block_size as usize],
        }
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        Self { buffer: b.to_vec() }
    }

    pub fn i32_mut(&self, offset: usize) -> i32 {
        let bytes = self.buffer[offset..offset + 4]
            .try_into()
            .expect("index out of bounds for get_int");
        i32::from_be_bytes(bytes)
    }

    pub fn i32(&mut self, offset: usize, n: i32) {
        let bytes = n.to_be_bytes();
        self.buffer[offset..offset + 4].copy_from_slice(&bytes);
    }

    pub fn bytes(&self, offset: usize) -> Vec<u8> {
        let length_bytes = self.buffer[offset..offset + 4]
            .try_into()
            .expect("index out of bounds for get_bytes length");
        let length = i32::from_be_bytes(length_bytes) as usize;
        let start = offset + 4;
        let end = start + length;
        self.buffer[start..end].to_vec()
    }

    pub fn bytes_mut(&mut self, offset: usize, b: &[u8]) {
        let length_bytes = (b.len() as i32).to_be_bytes();
        self.buffer[offset..offset + 4].copy_from_slice(&length_bytes);
        let start = offset + 4;
        let end = start + b.len();
        self.buffer[start..end].copy_from_slice(b);
    }

    pub fn string(&self, offset: usize) -> String {
        let b = self.bytes(offset);
        match String::from_utf8(b) {
            Ok(s) => s,
            Err(_) => String::from("�"),
        }
    }

    pub fn string_mut(&mut self, offset: usize, s: &str) {
        let b = s.as_bytes();
        self.bytes_mut(offset, b);
    }

    pub fn max_length(str_size: usize) -> usize {
        4 + str_size
    }

    pub fn contents(&self) -> &[u8] {
        &self.buffer
    }

    pub fn contents_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn size(&self) -> usize {
        self.buffer.len()
    }
}
