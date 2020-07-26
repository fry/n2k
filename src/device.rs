use core::fmt::Debug;

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::hal::can::{Receiver, Transmitter};
use crate::{Frame, Handler, Id, IdError, Message, GLOBAL_ADDRESS};

const CB_TP_BAM: u8 = 0x40; // Control byte indicating TP_BAM

const PGN_TP_CM: u32 = 0x00ec00; // 60416 - ISO Transport Protocol, Connection Management - RTS group
const PGN_TP_DT: u32 = 0x00eb00; // 60160 - ISO Transport Protocol, Data Transfer

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeviceError {
    CouldNotOpenBus,
    CouldNotSendMessage,
    InvalidId(IdError),
}

impl From<IdError> for DeviceError {
    fn from(error: IdError) -> Self {
        DeviceError::InvalidId(error)
    }
}

pub type Result<T> = core::result::Result<T, DeviceError>;

pub struct Device<T: Receiver + Transmitter> {
    bus: T,
    handlers: Vec<Box<dyn Handler>>,
    address: u8,
}

impl<T, E, F> Device<T>
where
    T: Receiver<Error = E, Frame = F> + Transmitter<Error = E, Frame = F>,
    F: crate::hal::can::Frame,
    E: Debug,
{
    pub fn new(bus: T) -> Self {
        Device {
            bus: bus,
            handlers: Vec::new(),
            address: 0,
        }
    }

    pub fn send(&mut self, message: &Message) -> Result<()> {
        let id = message.id();
        let data = message.data();
        let length = data.len();

        if length <= 8 {
            //TODO: Make sure it's not a fast packet
            let frame = &Frame::new(id, data);
            self.bus.transmit(frame);
            Ok(())
        } else {
            //calculate number of packets that will be sent
            let packets = (length / 7) + 1;
            // send broadcast announce message (BAM)
            let pgn = id.pgn();
            let priority = id.priority();
            let tp_cm_id = Id::new(priority, PGN_TP_CM, self.address, GLOBAL_ADDRESS)?;
            let d: [u8; 8] = [
                CB_TP_BAM,                    // Control Byte: TP_BAM
                (length & 0xff) as u8,        // message size LSB
                ((length >> 8) & 0xff) as u8, // message size MSB
                packets as u8,                // number of packets
                0xff,                         // maximun number of packets
                (pgn & 0xff) as u8,           // PGN LSB
                ((pgn >> 8) & 0xff) as u8,    // PGN
                ((pgn >> 16) & 0xff) as u8,   // PGN MSB
            ];

            let frame = &Frame::new(tp_cm_id, &d);
            self.bus.transmit(frame);

            // send packets
            let tp_dt_id = Id::new(priority, PGN_TP_DT, self.address, GLOBAL_ADDRESS)?;
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

                let frame = &Frame::new(tp_dt_id, &d);
                self.bus.transmit(frame);
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
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    use core::fmt::Debug;

    use crate::hal::can::{Receiver, Transmitter};
    use crate::{Device, Frame, Id, Message, Priority, GLOBAL_ADDRESS};

    struct MockCanBus<'a> {
        pub frames: Vec<&'a Frame>,
    }

    impl<'a> MockCanBus<'a> {
        pub fn new() -> Self {
            MockCanBus { frames: Vec::new() }
        }
    }

    impl<'a> Receiver for MockCanBus<'a> {
        type Frame = Frame;
        type Error = ();

        fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
            panic!();
        }
    }

    impl<'a> Transmitter for MockCanBus<'a> {
        type Frame = Frame;
        type Error = ();

        fn transmit(&mut self, frame: &Frame) -> nb::Result<Option<Self::Frame>, Self::Error> {
            self.frames.push(&frame);
            Ok(Option::None)
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

            device.send(&i.message).unwrap();

            let data = i.message.data();
            if data.len() <= 8 {
                // Single packet
            } else {
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
