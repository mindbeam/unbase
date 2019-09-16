
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from(number: u32) -> Self {
        Color {
            a: ((number & 0xff000000_u32 as u32) >> 24) as u8,
            r: ((number & 0xff0000) >> 16) as u8,
            g: ((number & 0xff00) >> 8) as u8,
            b: (number & 0xff) as u8,
        }
    }
}
