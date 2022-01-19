#![no_std]
#![no_main]

mod task_scheduler;
mod syscall_provider;
mod usb_hid;

#[cfg(not(debug_assertions))]
use panic_halt as _;

#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;
#[cfg(debug_assertions)]
use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use lazy_static::lazy_static;
use core::cell::RefCell;
use cortex_m::{asm, delay::Delay, interrupt, peripheral::syst::SystClkSource};
use cortex_m_rt::{entry, exception};
use stm32f1xx_hal::{device};
use cortex_m::interrupt::Mutex;
use task_scheduler::TaskScheduler;

lazy_static! {
    static ref TASK_SCHEDULER: Mutex<RefCell<TaskScheduler>> = Mutex::new(RefCell::new(TaskScheduler::new()));
}

#[entry]
fn main() -> ! {
    let p = device::Peripherals::take().unwrap();
    let cp = device::CorePeripherals::take().unwrap();

    #[cfg(debug_assertions)]
    let _ = hprintln!("OS init");

    // setup timer
    let mut syst = cp.SYST;
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(720_000); // 10ms
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    loop { }
}

#[exception]
fn SysTick() {
    // ctx switch
    interrupt::free(|cs| {
        TASK_SCHEDULER.borrow(cs).borrow_mut().tick();
    });
}
