use crate::PERIPHERALS;

use cortex_m::asm;
use stm32f1xx_hal::{pac::interrupt, gpio::{Input, Cr, CRL, CRH, Floating, PushPull, Output, GpioExt, gpioa, gpiod}, usb::{Peripheral, UsbBus}};
use usb_device::{class_prelude::{UsbBusAllocator}, device::{UsbDeviceBuilder, UsbVidPid, UsbDevice}, UsbError};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::{hid_class::HIDClass, descriptor::KeyboardReport};

static mut USB_BUS: Option<UsbBusAllocator<UsbBus<Peripheral>>> = None;
static mut USB_HID: Option<HIDClass<'static, UsbBus<Peripheral>>> = None;
static mut USB_DEVICE: Option<UsbDevice<'static, UsbBus<Peripheral>>> = None;

pub fn init(
    usb: stm32f1xx_hal::pac::USB, 
    pd6: gpiod::PD6<Input<Floating>>, 
    dcrl: &mut Cr<CRL, 'D'>, 
    pa11: gpioa::PA11<Input<Floating>>, 
    pa12: gpioa::PA12<Input<Floating>>, 
    acrh: &mut Cr<CRH, 'A'>
) {

    let usb = {
        let mut usb_en = pd6.into_push_pull_output(dcrl);
        usb_en.set_low();
    
        let mut usb_dp = pa12.into_push_pull_output(acrh);
        usb_dp.set_low();
        asm::delay(72_000_000 / 100);
    
        Peripheral {
            usb,
            pin_dm: pa11,
            pin_dp: usb_dp.into_floating_input(acrh),
        }
    };

    let usb_bus = UsbBus::new(usb);
    unsafe { USB_BUS = Some(usb_bus); }
    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    let usb_hid = HIDClass::new(&bus_ref, KeyboardReport::desc(), 10);
    let usb_device = UsbDeviceBuilder::new(&bus_ref, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Test Company")
        .product("ChocOS Keyboard")
        .serial_number("0001")
        .device_class(0xEF)
        .max_packet_size_0(64)
        .build();


    unsafe {
        USB_HID = Some(usb_hid);
        USB_DEVICE = Some(usb_device);
    }

    unsafe {
        stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USB_HP_CAN_TX);
        stm32f1xx_hal::pac::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USB_LP_CAN_RX0);
    }
}

#[interrupt]
fn USB_HP_CAN_TX() {
    usb_interrupt();
}

#[interrupt]
fn USB_LP_CAN_RX0() {
    usb_interrupt();
}

fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let usb_hid = unsafe { USB_HID.as_mut().unwrap() };
    if !usb_dev.poll(&mut [usb_hid]) {
        return;
    }

    let mut buf = [0u8; 64];
    match usb_hid.pull_raw_output(&mut buf) {
        Ok(_size) => {
            // TODO
        },
        Err(UsbError::InvalidEndpoint) => {

        },
        Err(UsbError::WouldBlock) => {
            return;
        },
        n => unreachable!()
    }
}
