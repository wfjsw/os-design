
use cortex_m::{asm, peripheral::NVIC};
// use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::{pac::interrupt, gpio::{Input, Cr, CRL, CRH, Floating, gpioa, gpiod}, usb::{Peripheral, UsbBus}, rcc::Clocks};
use usb_device::{class_prelude::{UsbBusAllocator}, device::{UsbDeviceBuilder, UsbVidPid, UsbDevice}, UsbError};
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::{hid_class::HIDClass, descriptor::KeyboardReport};

static mut USB_BUS: Option<UsbBusAllocator<UsbBus<Peripheral>>> = None;
static mut USB_HID: Option<HIDClass<'static, UsbBus<Peripheral>>> = None;
static mut USB_DEVICE: Option<UsbDevice<'static, UsbBus<Peripheral>>> = None;

pub fn init(
    sysclk: u32,
    usb: stm32f1xx_hal::pac::USB, 
    pd6: gpiod::PD6<Input<Floating>>, 
    dcrl: &mut Cr<CRL, 'D'>, 
    pa11: gpioa::PA11<Input<Floating>>, 
    pa12: gpioa::PA12<Input<Floating>>, 
    acrh: &mut Cr<CRH, 'A'>
) {

    let usb = {
        let mut usb_en = pd6.into_push_pull_output(dcrl);
        let mut usb_dp = pa12.into_push_pull_output(acrh);

        usb_en.set_high();
        asm::delay(sysclk / 10);
        
        usb_en.set_low();
    
        usb_dp.set_low();
        asm::delay(sysclk / 100);
    
        Peripheral {
            usb,
            pin_dm: pa11,
            pin_dp: usb_dp.into_floating_input(acrh),
        }
    };

    let usb_bus = UsbBus::new(usb);
    unsafe { USB_BUS = Some(usb_bus); }
    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    let usb_hid = HIDClass::new(bus_ref, KeyboardReport::desc(), 10);
    let usb_device = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27dd))
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

    cortex_m::interrupt::free(|cs| {
        unsafe {
            stm32f1xx_hal::pac::NVIC::unmask(interrupt::USB_HP_CAN_TX);
            stm32f1xx_hal::pac::NVIC::unmask(interrupt::USB_LP_CAN_RX0);
        }
    });

    // asm::wfi();
}

pub fn send_msg(msgid: u8) -> Result<usize, UsbError> {
    let usb_hid = unsafe { USB_HID.as_ref().unwrap() };

    usb_hid.push_raw_input(&msgid.to_ne_bytes())
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
    // let _ = hprintln!("USB_INTERRUPT");
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let usb_hid = unsafe { USB_HID.as_mut().unwrap() };
    let poll_result = usb_dev.poll(&mut [usb_hid]);
    // let _ = hprintln!("USB_POLL: {}", poll_result);
    // let _ = hprintln!("USB_STATE: {:?}", usb_dev.state());
    if !poll_result {
        return;
    }

    let mut buf = [0u8; 64];
    let data = usb_hid.pull_raw_output(&mut buf);

    match data {
        Ok(size) => {
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
