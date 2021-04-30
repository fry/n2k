#![no_std]

pub const GLOBAL_ADDRESS: u8 = 0xff;

use embedded_hal_can as hal_can;

mod bus;
pub use bus::{Bus, BusError};

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

mod frame;
pub use frame::CanFrame;
