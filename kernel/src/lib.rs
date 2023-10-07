#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod allocator;
pub mod apic;
pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod logger;
pub mod memory;
pub mod serial;
pub mod task;

#[cfg(test)]
use core::panic::PanicInfo;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    BootInfo,
};

#[cfg(test)]
use bootloader_api::entry_point;
use memory::BootInfoFrameAllocator;
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

pub fn init(boot_info: &'static mut BootInfo) {
    logger::init(
        boot_info
            .framebuffer
            .as_mut()
            .expect("Failed to get framebuffer"),
    );

    gdt::init();
    interrupts::init();
    unsafe { interrupts::PICS.lock().initialize() };

    x86_64::instructions::interrupts::enable();

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("Failed to get memory offset"),
    );

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Failed to initialization failed");

    apic::init(boot_info.rsdp_addr)
}

pub fn htl_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub mod tests {
    use core::panic::PanicInfo;

    use crate::{htl_loop, serial_print, serial_println};

    pub trait Testable {
        fn run(&self) -> ();
    }

    impl<T> Testable for T
    where
        T: Fn(),
    {
        fn run(&self) -> () {
            serial_print!("{} ...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
    }

    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("Running {} tests", tests.len());
        for test in tests {
            test.run();
        }
    }

    pub fn panic_handler(info: &PanicInfo) -> ! {
        serial_println!("[failed]");
        serial_println!("Error: {}\n", info);
        exit_qemu(QemuExitCode::Failed);
        htl_loop();
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u32)]
    pub enum QemuExitCode {
        Success = 0x10,
        Failed = 0x11,
    }

    pub fn exit_qemu(exit_code: QemuExitCode) {
        use x86_64::instructions::port::Port;

        unsafe {
            let mut port = Port::new(0xf4);
            port.write(exit_code as u32);
        }
    }
}

#[cfg(test)]
entry_point!(test_kernal_main, config = &BOOTLOADER_CONFIG);

#[cfg(test)]
fn test_kernal_main(boot: &'static mut BootInfo) -> ! {
    init(boot);
    test_main();
    htl_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    tests::panic_handler(info)
}
