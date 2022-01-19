#![no_std]
#![no_main]

mod flasher;

use stm32f1xx_hal::gpio::GpioExt;

use core::panic::PanicInfo;
use core::sync::atomic::{self, Ordering};

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;
#[cfg(debug_assertions)]
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::{asm, delay::Delay};
use cortex_m_rt::entry;
use stm32f1xx_hal::{device, prelude::*};

pub const OS_ADDR: u32 = 0x0800_0400;

#[entry]
fn main() -> ! {
    let p = device::Peripherals::take().unwrap();
    let cp = device::CorePeripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let mut flash = p.FLASH.constrain();

    // Initialize clocks
    let clocks = rcc.cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .hclk(72.mhz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .freeze(&mut flash.acr);

    let mut scb = cp.SCB;
    let mut gpioa = p.GPIOA.split();
    

    #[cfg(debug_assertions)]
    let _ = hprintln!("Bootloader init");

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
            go_bootloader(p, clocks);
        }
    }

    
    // wait
    let mut sleeper = Delay::new(cp.SYST, 72_000_000);
    sleeper.delay_ms(1000);
    sleeper.free();

    // assume GPIOA1 is the bootloader button

    let btn = gpioa.pa0.into_pull_up_input(&mut gpioa.crl);
    if btn.is_high() {
        #[cfg(debug_assertions)]
        let _ = hprintln!("Button pressed, jumping to isp");

        // TODO: get rid of this unsafe
        // 
        // use of partially moved value: `p`
        // partial move occurs because `p.GPIOA` has type `GPIOA`, which does not implement the `Copy` trait rustc(E0382)
        // main.rs(40, 29): `p.GPIOA` partially moved due to this method call
        // gpio.rs(120, 14): this function takes ownership of the receiver `self`, which moves `p.GPIOA`
        unsafe { go_bootloader(p, clocks); }
    }


    // boot usercode
    #[cfg(debug_assertions)]
    let _ = hprintln!("Jumping to user code");

    boot(&mut scb, OS_ADDR as *const u32);
}

fn go_bootloader(p: stm32f1xx_hal::pac::Peripherals, clocks: stm32f1xx_hal::rcc::Clocks) -> ! {
    #[cfg(debug_assertions)]
    let _ = hprintln!("Init flasher");

    loop {}
    // let flash = flasher::IspFlash::new(p, clocks);
    // flash.take_control();
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



#[cfg(not(debug_assertions))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // House is on fire!

    unsafe {
        // set flasher flag
        core::ptr::write_volatile(0x2000_FFF0 as *mut [u8; 8], [0xAD, 0x10, 0x07, 0xB0, 0x00, 0x00, 0x00, 0xE2]);

        // reset the board
        // See also: https://developer.arm.com/documentation/dui0552/a/Cihehdge
        let cp = device::CorePeripherals::steal();
        cp.SCB.aircr.write((0x5FA << 16) | (1 << 2));
    }

    loop {
        // wait till reset
    }
}
