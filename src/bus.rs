use crate::can::Device;

pub struct Bus<T: Device> {
    device: T,
}

impl<T> Bus<T>
where
    T: Device,
{
    pub fn new(device: T) -> Bus<T> {
        Bus { device }
    }
}
