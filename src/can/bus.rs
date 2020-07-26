use super::Frame;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BusError {
    CouldNotOpenBus,
    CouldNotSendMessage,
}

pub type Result<T> = core::result::Result<T, BusError>;

pub trait Bus{
    fn open(&self) -> Result<()>;
    fn send(&mut self, frame:Frame) -> Result<()>;
}