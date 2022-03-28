#![no_std]
#![no_main]

#[macro_use]
mod stdlib;

use core::{arch::asm, panic::PanicInfo};

entry!(main);

fn main() -> ! {
    

    loop {
        unsafe { asm!("wfi"); }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // House is on fire!

    unsafe {
        loop {
            asm!("wfi");
        }
    }
}
