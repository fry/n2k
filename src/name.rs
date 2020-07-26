/*
Arbitrary address bit
Industry group, length 3 bits
Vehicle system instance, length 4 bits
Vehicle system, length 7 bits
Reserved bit
Function, length 8 bits
Function instance, length 5 bits
ECU instance, length 3 bits
Manufacturer code, length 11 bits
Identity number, length 21 bits
*/

pub struct Name {
    name: u64,
}

impl Name {
    pub fn arbitrary_address(&self) -> bool{
        //TODO: implement
        true
    }

    pub fn industry_group(&self) -> u8 {
        //TODO: implement
        0
    }

    pub fn vehicle_system_instance(&self) -> u8 {
        //TODO: implement
        0
    }

    pub fn vehicle_system(&self) -> u8 {
        //TODO: implement
        0
    }

    pub fn function_instance(&self) -> u8 {
        //TODO: implement
        0
    }

    pub fn function(&self) -> u8 {
        //TODO: implement
        0
    }

    pub fn ecu_instance(&self) -> u8 {
        //TODO: implement
        0
    }

    pub fn manufacturer_code(&self) -> u16 {
        //TODO: implement
        0
    }

    pub fn identity_number(&self) -> u32 {
        //TODO: implement
        0
    }
}
