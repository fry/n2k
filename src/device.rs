use core::fmt::Debug;

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::hal::can::{Frame, Receiver, Transmitter};
use crate::{Handler, Id, IdError, Message, GLOBAL_ADDRESS};

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

pub struct Device<Rx, Tx> {
    rx: Rx,
    tx: Tx,
    handlers: Vec<Box<dyn Handler>>,
    address: u8,
}

impl<Rx, Tx> Device<Rx, Tx>
where
    Rx: Receiver,
    Tx: Transmitter,
{
    pub fn new(rx: Rx, tx: Tx) -> Self {
        Device {
            rx: rx,
            tx: tx,
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
            let frame = &Tx::Frame::new_extended(id.value(), data);
            self.tx.transmit(frame);
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

            let frame = &Tx::Frame::new_extended(tp_cm_id.value(), &tp_cm_id_data);
            self.tx.transmit(frame);

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

                let frame = &Tx::Frame::new_extended(tp_dt_id.value(), &tp_dt_data);
                self.tx.transmit(frame);
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

    use crate::hal::can::{Frame, Receiver, Transmitter};
    use crate::{Device, Id, Message, Priority, GLOBAL_ADDRESS};

    use core::convert::TryFrom;
    use core::fmt::Debug;
        
    /// A CAN data or remote frame.
    #[derive(Clone, Debug)]
    pub struct CanFrame {
        id: Id,
        dlc: usize,
        data: [u8; 8],
    }
    
    impl CanFrame {
        /// Creates a new data frame.
        pub fn new(id: Id, data: &[u8]) -> Self {
    
            let mut frame = Self {
                id,
                dlc: data.len(),
                data: [0; 8],
            };
            frame.data[0..data.len()].copy_from_slice(data);
            frame
        }
    
        /// Returns the frame identifier.
        fn id(&self) -> Id {
            self.id
        }
    }
    
    impl crate::hal::can::Frame for CanFrame
    {
        /// Creates a new frame with a standard identifier.
        fn new_standard(_id: u32, _data: &[u8]) -> Self {
            panic!("NMEA 2000 only supports extended frames")
        }
    
        /// Creates a new frame with an extended identifier.
        fn new_extended(id: u32, data: &[u8]) -> Self {
            Self::new(Id::try_from(id).unwrap(), data)
        }
    
        /// Marks the frame as a remote frame with configurable data length code (DLC).
        ///
        /// Remote frames do not contain any data, even if the frame was created with a
        /// non-empty data buffer.
        fn with_rtr(&mut self, _dlc: usize) -> &mut Self {
            panic!("NMEA 2000 only supports extended frames")
        }
    
        /// Returns true if this frame is an extended frame
        fn is_extended(&self) -> bool {
            true
        }
    
        /// Returns true if this frame is a standard frame
        fn is_standard(&self) -> bool {
            false
        }
    
        /// Returns true if this frame is a remote frame
        fn is_remote_frame(&self) -> bool {
            false
        }
    
        /// Returns true if this frame is a data frame
        fn is_data_frame(&self) -> bool {
            !self.is_remote_frame()
        }
    
        /// Returns the frame identifier.
        fn id(&self) -> u32 {
            self.id.value()
        }
    
        /// Returns the data length code (DLC) which is in the range 0..8.
        ///
        /// For data frames the DLC value always matches the lenght of the data.
        /// Remote frames no not carry any data, yet the DLC can be greater than 0.
        fn dlc(&self) -> usize {
            self.dlc
        }
    
        /// Returns the frame data (0..8 bytes in length).
        fn data(&self) -> &[u8] {
            if self.is_data_frame() {
                &self.data[0..self.dlc]
            } else {
                &[]
            }
        }
    }

    struct MockCanReceiver {
    }

    struct MockCanTransmitter {
        pub frames: Vec<CanFrame>,
    }

    impl MockCanTransmitter {
        pub fn new() -> Self {
            MockCanTransmitter { frames: Vec::new() }
        }
    }

    impl Receiver for MockCanReceiver {
        type Frame = CanFrame;
        type Error = ();

        fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
            panic!();
        }
    }

    impl Transmitter for MockCanTransmitter {
        type Frame = CanFrame;
        type Error = ();

        fn transmit(&mut self, frame: &CanFrame) -> nb::Result<Option<Self::Frame>, Self::Error> {
            self.frames.push(frame.clone());
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
            let rx = MockCanReceiver{};
            let tx = MockCanTransmitter::new();
            let mut device = Device::new(rx, tx);

            device.send(&i.message).unwrap();

            let data = i.message.data();
            if data.len() <= 8 {
                // Single packet
            } else {
                // Multipacket
                for b in 0..data.len() {
                    let frame = (b / 7) + 1;
                    let index = b - ((frame - 1) * 7) + 1;
                    assert_eq!(device.tx.frames[frame].data()[index], data[b])
                }
            }
        }
    }
}
