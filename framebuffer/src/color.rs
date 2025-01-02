pub const LOG: Color = Color::new(255, 255, 255);
pub const INFO: Color = Color::new(160, 160, 160);
pub const ERROR: Color = Color::new(255, 0, 0);
pub const OK: Color = Color::new(0, 255, 100);
pub const CAPTION: Color = Color::new(255, 255, 102);
pub const BACKGROUND: Color = Color::new(0, 0, 0);

#[derive(Copy, Clone, Debug, Default)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Color {
        Color { red, green, blue }
    }
}

impl Color {
    pub fn red(&self) -> u8 {
        self.red
    }
    pub fn green(&self) -> u8 {
        self.green
    }
    pub fn blue(&self) -> u8 {
        self.blue
    }
}
