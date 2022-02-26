use core::marker::PhantomData;
use core::ops::Deref;
use cortex_m::asm;

use crate::CORE_PERIPHERALS;

#[allow(dead_code)]
pub struct MPU {
    _marker: PhantomData<*const ()>,
}

impl MPU {
    pub unsafe fn arm() {
        let cp = CORE_PERIPHERALS.as_mut().unwrap();
        let mpu = cp.MPU.deref();
        mpu.ctrl.modify(|b| b | 1);
        asm::dsb();
    }

    pub unsafe fn disarm() {
        let cp = CORE_PERIPHERALS.as_mut().unwrap();
        let mpu = cp.MPU.deref();
        mpu.ctrl.modify(|b| b & !1);
        asm::dsb();
    }


}
