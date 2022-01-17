#![no_std]
#![no_main]

mod flasher;

use stm32f1xx_hal::gpio::GpioExt;

#[cfg(not(debug_assertions))]
use panic_halt as _;

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;
#[cfg(debug_assertions)]
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::{asm, delay::Delay};
use cortex_m_rt::entry;
use stm32f1xx_hal::{device};

pub const OS_ADDR: u32 = 0x0808_0000;

#[entry]
fn main() -> ! {
    let p = device::Peripherals::take().unwrap();
    let cp = device::CorePeripherals::take().unwrap();
    let mut scb = cp.SCB;
    let mut gpioa = p.GPIOA.split();
    let mut sleeper = Delay::new(cp.SYST, 72_000_000);

    #[cfg(debug_assertions)]
    let _ = hprintln!("Bootloader init");
    
    // wait for 1 sec
    sleeper.delay_ms(150);
    sleeper.free();

    // assume GPIOA1 is the bootloader button

    let btn = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
    if btn.is_high() {
        #[cfg(debug_assertions)]
        let _ = hprintln!("Button pressed, jumping to isp");
        go_bootloader();
    }

    // check SRAM flash flag
    unsafe {
        let flag = core::ptr::read_volatile(0x2000_FFF0 as *const [u8; 8]);
        // let _ = hprintln!("flag: {:?}", flag);
        // Flag: 0xB00710AD 0xE2000000 
        if flag == [0xAD, 0x10, 0x07, 0xB0, 0x00, 0x00, 0x00, 0xE2] {
            // clear the flag
            core::ptr::write_volatile(0x2000_FFF0 as *mut [u8; 8], [0x00; 8]);
            #[cfg(debug_assertions)]
            let _ = hprintln!("SRAM flag set, jumping to isp");
            go_bootloader();
        }
    }

    // boot usercode
    #[cfg(debug_assertions)]
    let _ = hprintln!("Jumping to user code");
    boot(&mut scb, OS_ADDR as *const u32);
}

fn go_bootloader() -> ! {
    #[cfg(debug_assertions)]
    let _ = hprintln!("Init flasher");
    loop {
        // your code goes here
    }
}

// Jump to the user application code
fn boot(scb: &mut cortex_m::peripheral::SCB, vtable: *const u32) -> ! {
    #[cfg(debug_assertions)]
    let _ = hprintln!("Booting");

    unsafe {
        scb.vtor.write(vtable as u32);
        asm::bootload(vtable);
    }
}
