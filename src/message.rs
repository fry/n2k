extern crate alloc;
use alloc::boxed::Box;

use crate::Id;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MessageError {
    Max255Bytes,
}

pub type Result<T> = core::result::Result<T, MessageError>;

pub struct Message {
    id: Id,
    data: Box<[u8]>,
}

impl Message {
    pub fn new(id: Id, data: Box<[u8]>) -> Result<Self>{
        if data.len() > 255 {
            return Err(MessageError::Max255Bytes)
        }
        
        Ok(Message{
            id: id,
            data: data
        })
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
