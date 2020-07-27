use core::convert::TryFrom;
use core::fmt::Debug;

use crate::Id;

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
    fn new_standard(id: u32, data: &[u8]) -> Self {
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
    fn with_rtr(&mut self, dlc: usize) -> &mut Self {
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
