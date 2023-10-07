use crate::println;
use bootloader_api::info::Optional;

pub fn init(rsdp_addr: Optional<u64>) {
    println!("LOADING ACPI");

    if let Optional::Some(_addr) = rsdp_addr {
        println!("Init acpi")
    } else {
        // most likly a BIOS system
        println!("No RSDP addrs was found. running is BIOS? or not reported.")
    }
}
