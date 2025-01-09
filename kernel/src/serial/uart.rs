use core::fmt;

use crate::{io, retry_until_ok};

use super::{LineStsFlags, WouldBlockError};

/// I/O port-mapped UART.
///
/// Common port mappings:
/// | COM Port | IO Port  |
/// |----------|----------|
/// | COM1     | 0x3F8    |
/// | COM2     | 0x2F8    |
/// | COM3     | 0x3E8    |
/// | COM4     | 0x2E8    |
/// | COM5     | 0x5F8    |
/// | COM6     | 0x4F8    |
/// | COM7     | 0x5E8    |
/// | COM8     | 0x4E8    |
#[derive(Debug)]
pub struct SerialPort(u16 /* base port */);

impl SerialPort {
    /// Base port.
    fn port_base(&self) -> u16 {
        self.0
    }

    /// Data port.
    ///
    /// Read and write.
    fn port_data(&self) -> u16 {
        self.port_base()
    }

    /// Interrupt enable port.
    ///
    /// Write only.
    fn port_int_en(&self) -> u16 {
        self.port_base() + 1
    }

    /// Fifo control port.
    ///
    /// Write only.
    fn port_fifo_ctrl(&self) -> u16 {
        self.port_base() + 2
    }

    /// Line control port.
    ///
    /// Write only.
    fn port_line_ctrl(&self) -> u16 {
        self.port_base() + 3
    }

    /// Modem control port.
    ///
    /// Write only.
    fn port_modem_ctrl(&self) -> u16 {
        self.port_base() + 4
    }

    /// Line status port.
    ///
    /// Read only.
    fn port_line_sts(&self) -> u16 {
        self.port_base() + 5
    }
}

impl SerialPort {
    /// Creates a new serial port interface on the given I/O base port.
    ///
    /// # Safety
    /// Caller must ensure that the given base address
    /// really points to a serial port device and that the caller has the necessary rights
    /// to perform the I/O operation.
    pub const unsafe fn new(base: u16) -> SerialPort {
        Self(base)
    }

    /// Initializes the serial port.
    ///
    /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            io::outb(self.port_int_en(), 0x00);

            // Enable DLAB
            io::outb(self.port_line_ctrl(), 0x80);

            // Set maximum speed to 38400 bps by configuring DLL and DLM
            io::outb(self.port_data(), 0x03);
            io::outb(self.port_int_en(), 0x00);

            // Disable DLAB and set data word length to 8 bits
            io::outb(self.port_line_ctrl(), 0x03);

            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 14 bytes
            io::outb(self.port_fifo_ctrl(), 0xc7);

            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            io::outb(self.port_modem_ctrl(), 0x0b);

            // Enable interrupts
            io::outb(self.port_int_en(), 0x01);
        }
    }

    fn line_sts(&mut self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(io::inb(self.port_line_sts())) }
    }
}

impl SerialPort {
    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        match data {
            8 | 0x7F => {
                self.send_raw(8);
                self.send_raw(b' ');
                self.send_raw(8);
            }
            data => {
                self.send_raw(data);
            }
        }
    }

    /// Sends a raw byte on the serial port, intended for binary data.
    pub fn send_raw(&mut self, data: u8) {
        retry_until_ok!(self.try_send_raw(data))
    }

    /// Tries to send a raw byte on the serial port, intended for binary data.
    pub fn try_send_raw(&mut self, data: u8) -> Result<(), WouldBlockError> {
        if self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {
            unsafe {
                io::outb(self.port_data(), data);
            }
            Ok(())
        } else {
            Err(WouldBlockError)
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
