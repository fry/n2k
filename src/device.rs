extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::can::{Bus, BusError};
use crate::Handler;
use crate::Message;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeviceError {
    CouldNotOpenBus,
    CouldNotSendMessage,
}

impl From<BusError> for DeviceError {
    fn from(error: BusError) -> Self {
        match error{
            BusError::CouldNotOpenBus => DeviceError::CouldNotOpenBus,
            BusError::CouldNotSendMessage => DeviceError::CouldNotSendMessage,
        }
    }
}

pub type Result<T> = core::result::Result<T, DeviceError>;

pub struct Device<T: Bus> {
    device: T,
    handlers: Vec<Box<dyn Handler>>,
}

impl<T> Device<T>
where
    T: Bus,
{
    pub fn new(device: T) -> Device<T> {
        Device {
            device: device,
            handlers: Vec::new(),
        }
    }

    pub fn open(&self) -> Result<()> {
        self.device.open()?;
        Ok(())
    }

    pub fn send(&self, message: &Message) {}

    pub fn register<H: Handler + 'static>(&mut self, handler: H) {
        //TODO: validate input
        self.handlers.push(Box::new(handler));
    }
}
