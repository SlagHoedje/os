#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

impl Color {
    pub fn from_ansi(code: u8, bright: bool) -> Option<Color> {
        if bright {
            match code {
                0 => Some(Color::DarkGray),
                1 => Some(Color::LightRed),
                2 => Some(Color::LightGreen),
                3 => Some(Color::Yellow),
                4 => Some(Color::LightBlue),
                5 => Some(Color::Pink),
                6 => Some(Color::LightCyan),
                7 => Some(Color::White),
                _ => None,
            }
        } else {
            match code {
                0 => Some(Color::Black),
                1 => Some(Color::Red),
                2 => Some(Color::Green),
                3 => Some(Color::Brown),
                4 => Some(Color::Blue),
                5 => Some(Color::Magenta),
                6 => Some(Color::Cyan),
                7 => Some(Color::LightGray),
                _ => None,
            }
        }
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub fn set_background(&mut self, background: Color) {
        self.0 = self.0 & 0x0f | (background as u8) << 4;
    }

    pub fn set_foreground(&mut self, foreground: Color) {
        self.0 = self.0 & 0xf0 | (foreground as u8);
    }
}