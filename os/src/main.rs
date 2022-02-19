#![no_std]
#![no_main]

#![feature(asm_const)]
#![feature(asm_sym)]
#![feature(naked_functions)]

mod structs;

mod task_scheduler;

#[macro_use]
mod syscall_provider;
mod usb_hid;

#[cfg(not(debug_assertions))]
use panic_halt as _;

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;
#[cfg(debug_assertions)]
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use lazy_static::lazy_static;
use core::{cell::RefCell, arch::asm};
use cortex_m::{interrupt, peripheral::syst::SystClkSource};
use cortex_m_rt::{entry, exception};
use stm32f1xx_hal::{device};
use cortex_m::interrupt::Mutex;
use task_scheduler::TaskScheduler;

static mut PERIPHERALS: Option<device::Peripherals> = None;
static mut CORE_PERIPHERALS: Option<device::CorePeripherals> = None;
static mut TASK_SCHEDULER: Option<TaskScheduler> = None;

#[entry]
fn main() -> ! {

    let p = device::Peripherals::take().unwrap();
    let cp = device::CorePeripherals::take().unwrap();

    unsafe {
        PERIPHERALS = Some(p);
        CORE_PERIPHERALS = Some(cp);
    }

    unsafe { 
        let scb = &mut CORE_PERIPHERALS.as_mut().unwrap().SCB;
        reset_scb(scb);
    }

    #[cfg(debug_assertions)]
    let _ = hprintln!("OS init");

    // setup timer
    unsafe {
        let syst = &mut CORE_PERIPHERALS.as_mut().unwrap().SYST;
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(720_000); // 10ms
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();
    }

    let task_scheduler = TaskScheduler::new();
    unsafe {
        TASK_SCHEDULER = Some(task_scheduler);

        // Init task scheduler
        TASK_SCHEDULER.as_mut().unwrap().init();
    }

    loop { }
}

// This fix VTOR to correct value so interrupts work flawlessly.
// not needed if not boot by SRAM
unsafe fn reset_scb(scb: &mut cortex_m::peripheral::SCB) {
    scb.vtor.write(0x2000_0000);
}

#[exception]
fn SysTick() {
    // ctx switch
    // this flags a PendSV interrupt
    stm32f1xx_hal::pac::SCB::set_pendsv();
}
