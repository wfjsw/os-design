use stm32f1xx_hal::prelude::_stm32_hal_flash_FlashExt;

pub struct IspFlash {
    // p: stm32f1xx_hal::pac::Peripherals,
    flash: stm32f1xx_hal::flash::Parts,
    clocks: stm32f1xx_hal::rcc::Clocks,
}

impl IspFlash {
    pub fn new(p: stm32f1xx_hal::pac::Peripherals, clocks: stm32f1xx_hal::rcc::Clocks) -> Self {
        let flash = p.FLASH.constrain();
        
        IspFlash {
            // p,
            flash,
            clocks,
        }
    }

    pub fn take_control(&self) -> ! {
        loop {
            // your code goes here
        }
    }

    pub fn wait_flash(&self) {
        // wait for flash to be ready
        // while self.flash.
    }

    pub fn unlock(&mut self) {
        self.wait_flash();
        // unlock flash
        // self.flash.keyr.write(|w| {
        //     w.key().bits(0x45670123).bits(0xCDEF89AB);
        // });
    }
}
