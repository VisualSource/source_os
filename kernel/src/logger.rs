use bootloader_api::info::FrameBuffer;
use conquer_once::spin::OnceCell;
use core::fmt::{self, Write};
use spin::mutex::SpinMutex;

use crate::{framebuffer::FrameBufferWriter, serial::Serial};

pub static WRITER: OnceCell<LockedWriter> = OnceCell::uninit();

pub struct LockedWriter {
    frame: Option<SpinMutex<FrameBufferWriter>>,
    serial: Option<SpinMutex<Serial>>,
}

impl LockedWriter {
    pub fn new(framebuffer: &'static mut FrameBuffer, stdo: bool, serial_output: bool) -> Self {
        let serial = match serial_output {
            true => Some(SpinMutex::new(unsafe { Serial::init() })),
            false => None,
        };

        let frame = match stdo {
            true => Some(SpinMutex::new(FrameBufferWriter::new(framebuffer))),
            false => None,
        };

        Self { serial, frame }
    }

    pub unsafe fn force_unlock(&self) {
        if let Some(serial) = &self.serial {
            unsafe { serial.force_unlock() };
        }
    }
}

pub fn init(framebuffer: &'static mut FrameBuffer) {
    WRITER.init_once(move || LockedWriter::new(framebuffer, true, true));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Some(writer) = WRITER.get() {
        if let Some(framebuffer) = &writer.frame {
            framebuffer.lock().write_fmt(args).expect("Failed to print");
        }
    }
}

#[doc(hidden)]
pub fn _serial_print(args: fmt::Arguments) {
    if let Some(wirter) = WRITER.get() {
        if let Some(serial) = &wirter.serial {
            serial
                .lock()
                .write_fmt(args)
                .expect("Failed to write to serial port");
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logger::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n",format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::logger::_serial_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt,"\n")));
    ($fmt:expr,$($arg:tt)*) => ($crate::serial_print!(concat!($fmt,"\n"), $($arg)*));
}
