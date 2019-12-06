use lazy_static::lazy_static;

use util::irq_lock::IrqLock;
use x86_64::port::Port;

/// The IRQ index for the first PIC
const PIC_1_OFFSET: u8 = 0x20;

/// The IRQ index for the second PIC
const PIC_2_OFFSET: u8 = 0x28;

lazy_static! {
    pub static ref PICS: IrqLock<ChainedPics> = IrqLock::new(ChainedPics::new());
}

/// A single PIC. This is never used standalone.
struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    /// Check if this PIC handles interrupt with id 'id'
    fn handles_interrupt(&self, id: u8) -> bool {
        self.offset <= id && id < self.offset + 8
    }

    /// Notify the PIC that the current interrupt is handled.
    fn end_of_interrupt(&mut self) {
        self.command.write(0x20);
    }
}

/// A representation of how the PIC's are set up on x86 hardware
pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    /// Creates and initializes 'ChainedPics'
    pub fn new() -> ChainedPics {
        let chained_pics = ChainedPics {
            pics: [
                Pic {
                    offset: PIC_1_OFFSET,
                    command: Port::new(0x20),
                    data: Port::new(0x21),
                },
                Pic {
                    offset: PIC_2_OFFSET,
                    command: Port::new(0xa0),
                    data: Port::new(0xa1),
                },
            ]
        };

        let wait_port: Port<u8> = Port::new(0x80);
        let wait = || wait_port.write(0);

        let saved_masks = (
            chained_pics.pics[0].data.read(),
            chained_pics.pics[1].data.read()
        );

        chained_pics.pics[0].command.write(0x11);
        wait();
        chained_pics.pics[1].command.write(0x11);
        wait();

        chained_pics.pics[0].data.write(PIC_1_OFFSET);
        wait();
        chained_pics.pics[1].data.write(PIC_2_OFFSET);
        wait();

        chained_pics.pics[0].data.write(4);
        wait();
        chained_pics.pics[1].data.write(2);
        wait();

        chained_pics.pics[0].data.write(1);
        wait();
        chained_pics.pics[1].data.write(1);
        wait();

        chained_pics.pics[0].data.write(saved_masks.0);
        chained_pics.pics[1].data.write(saved_masks.1);

        chained_pics
    }

    /// Checks if either of the PIC's contained in this 'ChainedPic' handles interrupt with id 'id'
    pub fn handles_interrupt(&self, id: u8) -> bool {
        self.pics.iter().any(|p| p.handles_interrupt(id))
    }

    /// Notify the correct PIC that the current interrupt with id 'id' is handled.
    fn end_of_interrupt(&mut self, id: u8) {
        if self.handles_interrupt(id) {
            if self.pics[1].handles_interrupt(id) {
                self.pics[1].end_of_interrupt();
            }

            self.pics[0].end_of_interrupt();
        }
    }
}