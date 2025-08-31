use x86_64::instructions::port::Port;
use core::fmt;

pub fn serial_init(base: u16) {
    unsafe {
        let mut port = Port::<u8>::new(base + 1); // COM1 + 1 = interrupt enable
        port.write(0x00); // disable interrupts

        let mut port = Port::<u8>::new(base + 3); // line control
        port.write(0x80); // enable DLAB

        let mut port = Port::<u8>::new(base + 0);
        port.write(0x03); // baud divisor low byte (38400 baud)

        let mut port = Port::<u8>::new(base + 1);
        port.write(0x00); // high byte

        let mut port = Port::<u8>::new(base + 3);
        port.write(0x03); // 8 bits, no parity, one stop bit

        let mut port = Port::<u8>::new(base + 2);
        port.write(0xC7); // enable FIFO

        let mut port = Port::<u8>::new(base + 4);
        port.write(0x0B); // IRQs enabled, RTS/DSR set
    }
}


pub struct Writer {
    line_status: Port::<u8>,
    tx: Port::<u8>,
}

impl Writer {
    pub fn new(base: u16) -> Self {
        serial_init(base);
        Self {
            line_status: Port::new(base + 5),
            tx: Port::new(base),
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                unsafe {
                    while (self.line_status.read() & 0x20) == 0 {} // wait until ready
                    self.tx.write(byte);
                }
            }
        }
    }

    fn new_line(&mut self) {
        unsafe {
            while (self.line_status.read() & 0x20) == 0 {} // wait until ready
            self.tx.write(b'\r');
        }
        unsafe {
            while (self.line_status.read() & 0x20) == 0 {} // wait until ready
            self.tx.write(b'\n');
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }

        }
    }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}