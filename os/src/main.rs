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

#[macro_use]
mod logger;

mod utils;

// #[cfg(not(debug_assertions))]
use panic_halt as _;

// #[cfg(debug_assertions)]
// use cortex_m_semihosting::hprintln;
// #[cfg(debug_assertions)]
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m::{peripheral::{syst::SystClkSource, scb::Exception}, interrupt::InterruptNumber};
use cortex_m_rt::{entry, exception};
use stm32f1::stm32f103::Interrupt;
use stm32f1xx_hal::{device, gpio::GpioExt, rcc::RccExt, prelude::*};
use task_scheduler::TaskScheduler;

static mut MPU: Option<cortex_m::peripheral::MPU> = None;
static mut TASK_SCHEDULER: Option<TaskScheduler> = None;
static mut TASK_SCHEDULER_INIT_READY: bool = false;

#[entry]
fn main() -> ! {

    let p = device::Peripherals::take().unwrap();
    let mut cp = device::CorePeripherals::take().unwrap();

    let rcc = p.RCC.constrain();
    let mut flash = p.FLASH.constrain();

    let clocks = rcc.cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .hclk(72.mhz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .adcclk(14.mhz())
        .freeze(&mut flash.acr);

    let main_freq = clocks.sysclk().0;

    assert!(clocks.usbclk_valid());

    unsafe {
        MPU = Some(cp.MPU);
    }

    unsafe { 
        let scb = &mut cp.SCB;
        reset_vtor(scb);
    }



    unsafe {
        cp.NVIC.set_priority(Interrupt::USB_HP_CAN_TX, 0);
        cp.NVIC.set_priority(Interrupt::USB_LP_CAN_RX0, 0);
    }

    // setup timer
    let syst = &mut cp.SYST;
    syst.set_clock_source(SystClkSource::External);
    syst.set_reload(clocks.sysclk().0 / 20); // 50ms
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    // setup usb
    let mut gpioa = p.GPIOA.split();
    let mut gpiod = p.GPIOD.split();
    let mut afio = p.AFIO.constrain();

    logger::init(
        p.USART1,
        gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
        gpioa.pa10.into_floating_input(&mut gpioa.crh),
        &mut afio.mapr,
        clocks,
    );


    // #[cfg(debug_assertions)]
    let _ = hprintln!("[ChocOS] Init: OS init");

    usb_hid::init(main_freq, p.USB, gpiod.pd6, &mut gpiod.crl, gpioa.pa11, gpioa.pa12, &mut gpioa.crh);

    // let _ = usb_hid::send_msg(0);

    let _ = hprintln!("[ChocOS] Init: Waiting for USB to ready");

    cortex_m::asm::delay(main_freq);

    // let _ = usb_hid::send_msg(1);
    let _ = hprintln!("[ChocOS] Init: Creating Task Scheduler instance");
    
    let task_scheduler = TaskScheduler::new();

    unsafe {
        TASK_SCHEDULER = Some(task_scheduler);

        let _ = hprintln!("[ChocOS] Init: Initializing Task Scheduler");

        // Init task scheduler
        TASK_SCHEDULER.as_mut().unwrap().init();
    }
}

fn sub_main() -> ! {

    // let task_scheduler = unsafe { TASK_SCHEDULER.as_mut().unwrap() };

    let _ = hprintln!("[ChocOS] Init: Loading demoapp into scheduler");

    // task_scheduler.create(0, 0x080200E0);

    syscall!(6, 0x080300A2, 0, 0);

    unsafe {TASK_SCHEDULER_INIT_READY = true};

    // let _ = usb_hid::send_msg(2);

    loop {
        // let _ = usb_hid::send_msg(3);

        cortex_m::asm::wfi(); // wait for interrupt
    }
}

// This fix VTOR to correct value so interrupts work flawlessly.
// not needed if not boot by SRAM
unsafe fn reset_vtor(scb: &mut cortex_m::peripheral::SCB) {
    // scb.vtor.write(0x2000_0000);
    scb.vtor.write(0x0801_0000);
}

#[exception]
unsafe fn SysTick() {
    // let _ = usb_hid::send_msg(4);
    let task_scheduler_opt = TASK_SCHEDULER.as_ref();
    if task_scheduler_opt.is_some() && TASK_SCHEDULER_INIT_READY {
        let task_scheduler = task_scheduler_opt.unwrap();
        if task_scheduler.is_activated {
            let _ = hprintln!("[Exception] SysTick: Set PendSV");
            // ctx switch
            // this flags a PendSV interrupt
            stm32f1xx_hal::pac::SCB::set_pendsv();
        }
    }

}
