use core::fmt;
use core::fmt::Error;

use lazy_static::lazy_static;
use volatile::Volatile;

use driver::vga::ansi::{AnsiParseIterator, AnsiSequencePart};
use driver::vga::color::{Color, ColorCode};
use util::irq_lock::IrqLock;

pub mod color;
pub mod ansi;

lazy_static! {
    /// A locked instance of `ScreenWriter` to be used by the kernel. This is so you can safely
    /// print everything to the vga buffer without data races or importing anything.
    pub static ref WRITER: IrqLock<ScreenWriter> = IrqLock::new(ScreenWriter::new());
}

/// A memory aligned struct to represent a character on the vga buffer. Contains the byte
/// representation of the character and the color.
#[derive(Copy, Clone)]
#[repr(C)]
struct ScreenChar {
    character: u8,
    color: ColorCode,
}

impl ScreenChar {
    /// Creates a new instance of `ScreenChar`
    pub fn new(character: u8, color: ColorCode) -> ScreenChar {
        ScreenChar {
            character,
            color,
        }
    }
}

/// A memory aligned struct to access the vga buffer safely. Protects the buffer from various
/// illegal actions, such as overflowing or corrupting the buffer.
#[repr(transparent)]
struct ScreenBuffer([[Volatile<ScreenChar>; 80]; 25]);

impl ScreenBuffer {
    /// Utility function to set a character in the vga buffer using an x and y coordinate.
    pub fn set(&mut self, x: u8, y: u8, character: ScreenChar) {
        self.0[y as usize][x as usize].write(character);
    }

    /// Utility function to get a character in the vga buffer using an x and y coordinate.
    pub fn get(&self, x: u8, y: u8) -> ScreenChar {
        self.0[y as usize][x as usize].read()
    }
}

/// An utility struct to write to the vga buffer without handling every byte manually. Also handles
/// ANSI escape code parsing through `driver::vga::ansi::AnsiParseIterator`.
pub struct ScreenWriter {
    buffer: &'static mut ScreenBuffer,
    cursor_position: (u8, u8),
    current_color: ColorCode,
}

impl ScreenWriter {
    // TODO: Mark unsafe?
    /// Creates a new instance of `ScreenWriter`. This internally also creates a new instance of
    /// `ScreenBuffer`. To avoid data races, this function should only be called once.
    pub fn new() -> ScreenWriter {
        ScreenWriter {
            buffer: unsafe { &mut *(0xb8000 as *mut ScreenBuffer) },
            cursor_position: (0, 0),
            current_color: ColorCode::new(Color::LightGray, Color::Black),
        }
    }

    /// Clears the screen using the current color and resets the cursor position to `(0, 0)`
    pub fn clear_screen(&mut self) {
        for x in 0..80 {
            for y in 0..25 {
                self.buffer.set(x, y, ScreenChar::new(b' ', self.current_color));
            }
        }

        self.cursor_position = (0, 0);
        self.update_cursor_position();
    }

    /// Writes a single byte to the screen. Also handles special escaped codes such as `\n` and
    /// `\r`. Does not handle ANSI escape codes.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\r' => self.cursor_position.0 = 0,
            b'\n' => {
                self.cursor_position.0 = 0;
                self.cursor_position.1 += 1;

                self.check_scroll_position();
            },
            _ => {
                let character = ScreenChar::new(byte, self.current_color);
                self.buffer.set(self.cursor_position.0, self.cursor_position.1, character);
                self.cursor_position.0 += 1;

                self.check_scroll_position();
            }
        }
    }

    /// Writes a string to the screen. A newline character is not automatically appended. This also
    /// handles any and all ANSI escape codes that might be present in the string.
    pub fn write_string(&mut self, string: &str) {
        let ansi_parse_iter = AnsiParseIterator::new(string);

        for part in ansi_parse_iter {
            match part {
                AnsiSequencePart::Text(text) => {
                    for byte in text.bytes() {
                        self.write_byte(byte);
                    }
                },
                AnsiSequencePart::SGR(sgr) => {
                    match sgr {
                        0 => self.current_color = ColorCode::new(Color::LightGray, Color::Black),
                        30..=37 => {
                            let color = Color::from_ansi(sgr - 30, false).unwrap();
                            self.current_color.set_foreground(color);
                        },
                        40..=47 => {
                            let color = Color::from_ansi(sgr - 40, false).unwrap();
                            self.current_color.set_background(color);
                        },
                        90..=97 => {
                            let color = Color::from_ansi(sgr - 90, true).unwrap();
                            self.current_color.set_foreground(color);
                        },
                        _ => (),
                    }
                }
            }
        }

        self.update_cursor_position();
    }

    /// Internal function to check and update the scroll position if necessary. Resets the x
    /// position and increases the y position when the right edge of the buffer is reached. Also
    /// scrolls the screen up when the bottom of the buffer is reached.
    fn check_scroll_position(&mut self) {
        if self.cursor_position.0 >= 80 {
            self.cursor_position.0 = 0;
            self.cursor_position.1 += 1;
        }

        if self.cursor_position.1 >= 25 {
            for y in 0..24 {
                for x in 0..80 {
                    self.buffer.set(x, y, self.buffer.get(x, y + 1));
                }
            }

            let blank = ScreenChar::new(b' ', self.current_color);

            for x in 0..80 {
                self.buffer.set(x, 24, blank);
            }

            self.cursor_position.1 -= 1;
        }
    }

    // TODO: Implement cursor (also need port io api)
    /// Updates VGA cursor position using ports on the cpu.
    fn update_cursor_position(&mut self) {

    }
}

impl fmt::Write for ScreenWriter {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.write_string(s);
        Ok(())
    }
}