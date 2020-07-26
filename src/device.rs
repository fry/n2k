extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::can::{Bus, BusError, Frame};
use crate::{Handler, Id, IdError, Message, Priority, GLOBAL_ADDRESS};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeviceError {
    CouldNotOpenBus,
    CouldNotSendMessage,
    InvalidId(IdError),
}

impl From<BusError> for DeviceError {
    fn from(error: BusError) -> Self {
        match error {
            BusError::CouldNotOpenBus => DeviceError::CouldNotOpenBus,
            BusError::CouldNotSendMessage => DeviceError::CouldNotSendMessage,
        }
    }
}

impl From<IdError> for DeviceError {
    fn from(error: IdError) -> Self {
        DeviceError::InvalidId(error)
    }
}

pub type Result<T> = core::result::Result<T, DeviceError>;

pub struct Device<T: Bus> {
    bus: T,
    handlers: Vec<Box<dyn Handler>>,
    address: u8,
}

impl<T> Device<T>
where
    T: Bus,
{
    pub fn new(bus: T) -> Device<T> {
        Device {
            bus: bus,
            handlers: Vec::new(),
            address: 0,
        }
    }

    pub fn open(&self) -> Result<()> {
        self.bus.open()?;
        Ok(())
    }

    pub fn send(&mut self, message: &Message) -> Result<()> {
        let id = message.id().value();
        let pgn = message.id().pgn();
        let data = message.data();
        let length = data.len();

        if length < 8 {
            //TODO: Make sure it's not a fast packet
            let mut d: [u8; 8] = [255; 8];
            for i in 0..length {
                d[i] = data[i];
            }
            let frame = Frame::new(id, length as u8, d);
            self.bus.send(frame)?;
            Ok(())
        } else {
            //calculate number of packets that will be sent
            let packets = (length / 7) + 1;

            // send broadcast announce message (BAM)
            let id = Id::new(Priority::Priority0, 60416, self.address, GLOBAL_ADDRESS)?;
            let d: [u8; 8] = [
                0x40,                         // Control Byte: 32=BAM
                (length & 0xff) as u8,        // message size LSB
                ((length >> 8) & 0xff) as u8, // message size MSB
                packets as u8,                // number of packets
                0xff,                         // maximun number of packets
                (pgn & 0xff) as u8,           // PGN LSB
                ((pgn >> 8) & 0xff) as u8,    // PGN
                ((pgn >> 16) & 0xff) as u8,   // PGN MSB
            ];

            let frame = Frame::new(id.value(), length as u8, d);
            self.bus.send(frame)?;

            // send packets
            let id = Id::new(Priority::Priority0, 60160, self.address, GLOBAL_ADDRESS)?;
            let mut count = 1;
            let mut index = 0;
            let mut remaining = length;
            let mut len;
            while remaining > 0 {
                len = remaining;
                if len > 7 {
                    len = 7;
                }
                remaining -= len;

                // fill data
                let mut d: [u8; 8] = [255; 8];

                d[0] = count;
                count += 1;
                for i in 0..len {
                    d[i + 1] = data[index];
                    index += 1;
                }

                let frame = Frame::new(id.value(), 8, d);
                self.bus.send(frame)?
            }

            Ok(())
        }
    }

    pub fn register<H: Handler + 'static>(&mut self, handler: H) {
        //TODO: validate input
        self.handlers.push(Box::new(handler));
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use crate::can::{Bus, BusError, Frame, Result};
    use crate::{Device, Handler, Id, IdError, Message, Priority, GLOBAL_ADDRESS};
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    struct MockCanBus {
        pub frames: Vec<Frame>,
    }

    impl MockCanBus {
        pub fn new() -> MockCanBus {
            MockCanBus { frames: Vec::new() }
        }
    }

    impl Bus for MockCanBus {
        fn open(&self) -> Result<()> {
            Ok(())
        }

        fn send(&mut self, frame: Frame) -> Result<()> {
            self.frames.push(frame);
            Ok(())
        }
    }

    #[test]
    fn device_send() {
        struct TestCase {
            message: Message,
        }
        let test_cases = [
            TestCase {
                message: Message::new(
                    Id::new(Priority::Priority0, 12345, 123, GLOBAL_ADDRESS).unwrap(),
                    Box::new([1, 2, 3, 4, 5, 6, 7]),
                )
                .unwrap(),
            },
            TestCase {
                message: Message::new(
                    Id::new(Priority::Priority0, 12345, 123, GLOBAL_ADDRESS).unwrap(),
                    Box::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]),
                )
                .unwrap(),
            },
        ];
        for i in &test_cases {
            let bus = MockCanBus::new();
            let mut device = Device::new(bus);
            device.open().unwrap();

            device.send(&i.message).unwrap();

            let data = i.message.data();
            if data.len() <= 8 {
                // Single packet
            }else{
                // Multipacket
                for b in 0..data.len() {
                    let frame = (b / 7) + 1;
                    let index = b - ((frame - 1) * 7) + 1;
                    assert_eq!(device.bus.frames[frame].data()[index], data[b])
                }
            }
        }
    }
}
