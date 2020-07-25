use crate::Id;

pub struct Message {
    id: Id,
    data: [u8],
}

impl Message {
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
