use core::fmt::{Error, Write};

use flagset::{flags, FlagSet};
use spin::Mutex;

use x86_64::port::Port;

pub const UART: Mutex<UART16550> = Mutex::new(UART16550::new(0x3F8));

flags! {
    enum LineStsFlags: u8 {
        InputFull = 1,
        OutputEmpty = 1 << 5,
    }
}

pub struct UART16550 {
    data: Port<u8>,
    int_en: Port<u8>,
    fifo_ctrl: Port<u8>,
    line_ctrl: Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts: Port<u8>,
}

impl UART16550 {
    pub const fn new(base: u16) -> UART16550 {
        UART16550 {
            data: Port::new(base),
            int_en: Port::new(base + 1),
            fifo_ctrl: Port::new(base + 2),
            line_ctrl: Port::new(base + 3),
            modem_ctrl: Port::new(base + 4),
            line_sts: Port::new(base + 5),
        }
    }

    pub fn init(&mut self) {
        // Disable interrupts
        self.int_en.write(0x00);

        // Enable DLAB
        self.line_ctrl.write(0x80);

        // Set maximum speed to 384000 bps
        self.data.write(0x03);
        self.int_en.write(0x00);

        // Disable DLAB
        self.line_ctrl.write(0x03);

        // Enable FIFO
        self.fifo_ctrl.write(0xC7);

        // Mark data terminal ready, enable auxiliary output #2
        self.modem_ctrl.write(0x0B);

        // Enable interrupts
        self.int_en.write(0x01);
    }

    fn line_sts(&mut self) -> FlagSet<LineStsFlags> {
        FlagSet::new_truncated(self.line_sts.read())
    }

    pub fn send_byte(&mut self, data: u8) {
        match data {
            8 | 0x7F => {
                while !self.line_sts().contains(LineStsFlags::OutputEmpty) {}
                self.data.write(8);
                while !self.line_sts().contains(LineStsFlags::OutputEmpty) {}
                self.data.write(b' ');
                while !self.line_sts().contains(LineStsFlags::OutputEmpty) {}
                self.data.write(8);
            }
            _ => {
                while !self.line_sts().contains(LineStsFlags::OutputEmpty) {}
                self.data.write(data);
            }
        }
    }
}

impl Write for UART16550 {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for byte in s.bytes() {
            self.send_byte(byte);
        }

        Ok(())
    }
}