use core::fmt::Write;

use stm32f1::stm32f103::USART1;
use stm32f1xx_hal::{gpio::{Input, Floating, gpioa, Alternate, PushPull}, serial::{Config, Tx, Rx}};
use stm32f1xx_hal::time::U32Ext;

static mut SERIAL_TX: Option<Tx<USART1>> = None;
static mut SERIAL_RX: Option<Rx<USART1>> = None;

pub fn init(
    usart1: USART1,
    tx: gpioa::PA9<Alternate<PushPull>>, 
    rx: gpioa::PA10<Input<Floating>>, 
    mapr: &mut stm32f1xx_hal::afio::MAPR,
    clocks: stm32f1xx_hal::rcc::Clocks,
) {

    let usart1Serial = stm32f1xx_hal::serial::Serial::usart1(
        usart1, 
        (tx, rx), 
        mapr, 
        Config::default().baudrate(9_600.bps()), 
        clocks
    );

    let (tx, rx) = usart1Serial.split();

    unsafe {
        SERIAL_TX = Some(tx);
        SERIAL_RX = Some(rx);
    }

}

pub fn hstdout_str(s: &str) {
    let mut tx = unsafe { SERIAL_TX.as_mut().unwrap() };
    let _ = tx.write_str(s);
}

pub fn hstdout_fmt(args: core::fmt::Arguments) {
    let mut tx = unsafe { SERIAL_TX.as_mut().unwrap() };
    let _ = tx.write_fmt(args).ok();
}

#[macro_export]
macro_rules! hprint {
    ($s:expr) => {
        $crate::logger::hstdout_str($s)
    };
    ($($tt:tt)*) => {
        $crate::logger::hstdout_fmt(format_args!($($tt)*))
    };
}

#[macro_export]
macro_rules! hprintln {
    () => {
        $crate::logger::hstdout_str("\n")
    };
    ($s:expr) => {
        $crate::logger::hstdout_str(concat!($s, "\n"))
    };
    ($s:expr, $($tt:tt)*) => {
        $crate::logger::hstdout_fmt(format_args!(concat!($s, "\n"), $($tt)*))
    };
}

