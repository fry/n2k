use super::frame::Frame;

pub trait Device{
    fn open(&self);
    fn send(&self, frame:Frame);
    fn receive(&self, frame:Frame);
}