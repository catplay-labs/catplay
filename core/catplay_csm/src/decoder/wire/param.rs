#[derive(Debug)]
pub struct CsmParam<'a> {
    pub id: u16,
    pub value: &'a [u8],
}

impl<'a> CsmParam<'a> {
    pub fn new(id: u16, value: &'a [u8]) -> Self {
        Self { id, value }
    }
}
