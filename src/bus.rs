use core::fmt::Debug;

use crate::hal_can::{Receiver, Transmitter};
use crate::CanFrame;
use crate::{Id, IdError, Message, GLOBAL_ADDRESS};

const CB_TP_BAM: u8 = 0x40; // Control byte indicating TP_BAM

const PGN_TP_CM: u32 = 0x00ec00; // 60416 - ISO Transport Protocol, Connection Management - RTS group
const PGN_TP_DT: u32 = 0x00eb00; // 60160 - ISO Transport Protocol, Data Transfer

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BusError {
    CouldNotOpenBus,
    CouldNotSendMessage,
    InvalidId(IdError),
}

impl From<IdError> for BusError {
    fn from(error: IdError) -> Self {
        BusError::InvalidId(error)
    }
}

pub type Result<T> = core::result::Result<T, BusError>;

pub struct Bus<T> {
    can: T,
    address: u8,
}

impl<T, E> Bus<T>
where
    E: core::fmt::Debug,
    T: Receiver<Frame = CanFrame, Error = E> + Transmitter<Frame = CanFrame, Error = E>,
{
    pub fn new(can: T) -> Self {
        Bus { can, address: 0 }
    }

    pub fn send(&mut self, message: &Message) -> Result<()> {
        let id = message.id();
        let data = message.data();
        let length = data.len();

        if length <= 8 {
            //TODO: Make sure it's not a fast packet
            let frame = CanFrame::new(id, data);
            self.transmit(&frame)?;
            Ok(())
        } else {
            //calculate number of packets that will be sent
            let packets = (length / 7) + 1;
            // send broadcast announce message (BAM)
            let pgn = id.pgn();
            let priority = id.priority();
            let tp_cm_id = Id::new(priority, PGN_TP_CM, self.address, GLOBAL_ADDRESS)?;
            let tp_cm_id_data = [
                CB_TP_BAM,                    // Control Byte: TP_BAM
                (length & 0xff) as u8,        // message size LSB
                ((length >> 8) & 0xff) as u8, // message size MSB
                packets as u8,                // number of packets
                0xff,                         // maximun number of packets
                (pgn & 0xff) as u8,           // PGN LSB
                ((pgn >> 8) & 0xff) as u8,    // PGN
                ((pgn >> 16) & 0xff) as u8,   // PGN MSB
            ];

            let frame = CanFrame::new(tp_cm_id, &tp_cm_id_data);
            self.transmit(&frame)?;

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
                let mut tp_dt_data = [255; 8];

                tp_dt_data[0] = count;
                count += 1;
                for i in 0..len {
                    tp_dt_data[i + 1] = data[index];
                    index += 1;
                }

                let frame = CanFrame::new(tp_dt_id, &tp_dt_data);
                self.transmit(&frame)?;
            }

            Ok(())
        }
    }

    fn transmit(&mut self, frame: &CanFrame) -> Result<()> {
        // TODO: revise this as it's not looking optimal or correct
        let result = self.can.transmit(frame);
        match result {
            Ok(None) => Ok(()),
            // A lower priority frame was replaced with our high priority frame.
            // Put the low priority frame back in the transmit queue.
            Ok(pending_frame) => {
                if let Some(f) = pending_frame {
                    self.transmit(&f)
                } else {
                    Ok(())
                }
            }
            Err(nb::Error::WouldBlock) => self.transmit(frame), // Need to retry
            _ => Err(BusError::CouldNotSendMessage),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::boxed::Box;
    use alloc::vec::Vec;

    use crate::hal_can::{Filter, Frame, Interface, Receiver, Transmitter};
    use crate::{Bus, Id, Message, Priority, GLOBAL_ADDRESS};

    use crate::frame::*;
    struct MockCan {
        pub frames: Vec<CanFrame>,
    }

    impl MockCan {
        pub fn new() -> Self {
            MockCan { frames: Vec::new() }
        }
    }

    struct MockFilter {}

    impl Filter for MockFilter {
        type Id = Id;

        fn from_id(_id: Self::Id) -> Self {
            panic!();
        }

        fn accept_all() -> Self {
            panic!();
        }

        fn from_mask(_mask: u32, _filter: u32) -> Self {
            panic!();
        }
    }

    impl Interface for MockCan {
        type Frame = CanFrame;
        type Id = Id;
        type Error = ();
        type Filter = MockFilter;
    }

    impl Receiver for MockCan {
        fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
            panic!();
        }

        fn set_filter(&mut self, _filter: Self::Filter) {
            panic!();
        }

        fn clear_filter(&mut self) {
            panic!();
        }
    }

    impl Transmitter for MockCan {
        fn transmit(&mut self, frame: &CanFrame) -> nb::Result<Option<Self::Frame>, Self::Error> {
            self.frames.push(frame.clone());
            Ok(Option::None)
        }
    }

    #[test]
    fn bus_send() {
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
            let can = MockCan::new();
            let mut bus = Bus::new(can);

            bus.send(&i.message).unwrap();

            let data = i.message.data();
            if data.len() <= 8 {
                // Single packet
            } else {
                // Multipacket
                for b in 0..data.len() {
                    let frame = (b / 7) + 1;
                    let index = b - ((frame - 1) * 7) + 1;
                    assert_eq!(bus.can.frames[frame].data().unwrap()[index], data[b])
                }
            }
        }
    }
}
