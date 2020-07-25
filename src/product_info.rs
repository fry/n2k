pub struct ProductInfo<'a> {
    n2k: u8,
    code: u8,
    model: &'a str,
    software: &'a str,
    version: &'a str,
    serial: &'a str,
    certification: u8,
    load: u8,
}

impl<'a> ProductInfo<'a> {
    pub fn new(
        n2k: u8,
        code: u8,
        model: &'a str,
        software: &'a str,
        version: &'a str,
        serial: &'a str,
        certification: u8,
        load: u8,
    ) -> ProductInfo<'a> {
        //TODO: validate parameters

        ProductInfo {
            n2k: n2k,
            code: code,
            model: model,
            software: software,
            version: version,
            serial: serial,
            certification: certification,
            load: load,
        }
    }
}
