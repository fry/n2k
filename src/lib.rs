#![no_std]

pub const GLOBAL_ADDRESS: u8 = 0xff;

pub mod can;

mod device;
pub use device::{Device, DeviceError};

mod handler;
pub use handler::Handler;

mod id;
pub use id::{Id, IdError, Priority};

mod message;
pub use message::Message;

mod name;
pub use name::Name;

mod product;
pub use product::Product;
