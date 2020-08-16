use crate::Message;

pub trait Handler {
    fn handle(&self, message: Message);
}
