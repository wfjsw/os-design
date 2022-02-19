#![no_std]
#![no_main]

// mod flasher;

use cortex_m::{interrupt::{free as critical_section}, asm::delay};
use stm32f1xx_hal::{pac::interrupt, gpio::{Input, Floating, PushPull, Output}, usb::{Peripheral, UsbBus}};

use stm32f1xx_hal::gpio::GpioExt;
use usb_device::{class_prelude::{UsbBusAllocator}, device::{UsbDeviceBuilder, UsbVidPid, UsbDevice}, UsbError};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::{hid_class::HIDClass, descriptor::KeyboardReport};

#[cfg(not(debug_assertions))]
use core::panic::PanicInfo;

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
    let _clocks = rcc.cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .hclk(72.mhz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .freeze(&mut flash.acr);

    let mut scb = cp.SCB;
    let mut gpioa = p.GPIOA.split();
    let mut gpiod = p.GPIOD.split();

    // debug in memory: set scb to 0x20000000 so interrupt handler would work properly
    unsafe { scb.vtor.write(0x2000_0000); }

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
            go_bootloader(flash, p.USB, &mut gpioa.crh, &mut gpiod.crl, gpioa.pa11, gpioa.pa12, gpiod.pd6);
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
        go_bootloader(flash, p.USB, &mut gpioa.crh, &mut gpiod.crl, gpioa.pa11, gpioa.pa12, gpiod.pd6);
    }


    // boot usercode
    #[cfg(debug_assertions)]
    let _ = hprintln!("Jumping to user code");

    boot(&mut scb, OS_ADDR as *const u32);
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

static mut USB_BUS: Option<UsbBusAllocator<UsbBus<Peripheral>>> = None;
static mut USB_HID: Option<HIDClass<'static, UsbBus<Peripheral>>> = None;
static mut USB_DEVICE: Option<UsbDevice<'static, UsbBus<Peripheral>>> = None;

// static usbBus: RefCell<Option<UsbBusAllocator<UsbBus<Peripheral>>>> = RefCell::new(None);
fn go_bootloader(_flash: stm32f1xx_hal::flash::Parts, usb: stm32f1xx_hal::pac::USB, crh: &mut stm32f1xx_hal::gpio::Cr<stm32f1xx_hal::gpio::CRH, 'A'>, crl: &mut stm32f1xx_hal::gpio::Cr<stm32f1xx_hal::gpio::CRL, 'D'>, pa11: stm32f1xx_hal::gpio::gpioa::PA11<Input<Floating>>, pa12: stm32f1xx_hal::gpio::gpioa::PA12<Input<Floating>>, pd6: stm32f1xx_hal::gpio::gpiod::PD6<Input<Floating>>) -> ! {
    #[cfg(debug_assertions)]
    let _ = hprintln!("Init flasher");

    let mut usb_en = pd6.into_push_pull_output(crl);
    usb_en.set_low();

    let mut usb_dp = pa12.into_push_pull_output(crh);
    usb_dp.set_low();
    asm::delay(72_000_000 / 100);

    let usb: Peripheral = Peripheral {
        usb,
        pin_dm: pa11,
        pin_dp: usb_dp.into_floating_input(crh),
    };

    let usb_bus = UsbBus::new(usb);
    unsafe { USB_BUS = Some(usb_bus); }
    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    let usb_hid = HIDClass::new(&bus_ref, KeyboardReport::desc(), 10);
    let usb_device = UsbDeviceBuilder::new(&bus_ref, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Test Company")
        .product("ChocOS Keyboard (Recovery)")
        .serial_number("0001")
        .device_class(0xEF)
        .max_packet_size_0(64)
        .build();

    unsafe {
        USB_HID = Some(usb_hid);
        USB_DEVICE = Some(usb_device);
    }

    // unsafe {
    //     // stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USB_LP_CAN_RX0);
    // }

    // usbBus.replace(Some(UsbBus::new(usb)));

    // critical_section(|cs| {
    //     USB_BUS.borrow(cs).replace(Some(UsbBus::new(usb)));
    //     let bus = USB_BUS.borrow(cs).borrow().as_ref().unwrap();

    //     let flasher = IspFlash::new(flash, &bus);
    //     FLASHER.borrow(cs).replace(Some(flasher));
    // });

    // unsafe {
    //     stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USB_HP_CAN_TX);
    //     stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USB_LP_CAN_RX0);
    // }

    loop {
        // asm::wfi();
        let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
        let usb_hid = unsafe { USB_HID.as_mut().unwrap() };
        if !usb_dev.poll(&mut [usb_hid]) {
            continue;
        }
        let mut buf = [0u8; 64];
        match usb_hid.pull_raw_output(&mut buf) {
            Ok(_size) => {
                // TODO
            },
            Err(UsbError::InvalidEndpoint) => {
    
            },
            Err(UsbError::WouldBlock) => {
                continue;
            },
            n => unreachable!()
        }
    

    }
    // let flash = flasher::IspFlash::new(p, clocks);
    // flash.take_control();
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

