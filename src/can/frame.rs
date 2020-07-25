pub struct Frame {
    id: u32,
    length: u8,
    data: [u8; 8]
}

impl Frame {
    pub fn new(id: u32, length: u8, data: [u8; 8]) -> Frame {
        //TODO: validate parameters
        Frame { id, length, data }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn length(&self) -> u8 {
        self.length
    }

    pub fn data(&self) -> [u8; 8] {
        self.data
    }
}
