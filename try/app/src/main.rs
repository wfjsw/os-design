#![no_std]
#![no_main]

// pick a panicking behavior
// use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{prelude::*};
use stm32f1xx_hal::gpio::GpioExt;
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::{peripheral::syst::SystClkSource};
use cortex_m_rt::entry;
use stm32f1xx_hal::{device};

#[entry]
fn main() -> ! {
    // asm::nop(); // To not have main optimize to abort in release mode, remove when you add code

    let p = device::Peripherals::take().unwrap();
    let mut cp = device::CorePeripherals::take().unwrap();
    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(72_000_000);
    syst.clear_current();
    syst.enable_counter();

    let mut rcc = p.RCC.constrain();

    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    let mut btn = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);

    let mut ledR = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
    let mut ledG = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let mut ledB = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);

    let mut state = 0;
    let mut pushed = false;

    loop {
        // your code goes here
        if !pushed && btn.is_low().unwrap() {
            state = (state + 1) % 5;
            pushed = true;
        } else if pushed && btn.is_high().unwrap() {
            pushed = false;
        }

        match state {
            0 => {
                ledR.set_high().unwrap();
                ledG.set_low().unwrap();
                ledB.set_low().unwrap();
            }
            1 => {
                ledR.set_low().unwrap();
                ledG.set_high().unwrap();
                ledB.set_low().unwrap();
            }
            2 => {
                ledR.set_low().unwrap();
                ledG.set_low().unwrap();
                ledB.set_high().unwrap();
            }
            3 => {
                ledR.set_low().unwrap();
                ledG.set_low().unwrap();
                ledB.set_low().unwrap();
            }
            4 => {
                ledR.set_high().unwrap();
                ledG.set_high().unwrap();
                ledB.set_high().unwrap();
            }
            _ => {}
        }
    }
}
