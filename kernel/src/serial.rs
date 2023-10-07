use core::fmt;

pub struct Serial(uart_16550::SerialPort);

impl Serial {
    pub unsafe fn init() -> Self {
        let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
        port.init();
        Self(port)
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_str(s).expect("Failed to write to serial port");
        Ok(())
    }
}
