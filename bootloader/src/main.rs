#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::{peripheral::syst::SystClkSource};
use cortex_m_rt::entry;
use stm32f1xx_hal::{device};

#[entry]
fn main() -> ! {
    // asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    let p = cortex_m::Peripherals::take().unwrap();
    let mut syst = p.SYST;
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(72_000_000);
    syst.clear_current();
    syst.enable_counter();

    

    loop {
        // your code goes here
    }
}
