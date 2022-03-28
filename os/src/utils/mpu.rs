use core::marker::PhantomData;
use core::ops::Deref;
use cortex_m::asm;

use crate::MPU;

// https://arxiv.org/pdf/1908.03638 
// 根 本 没 人 用

#[allow(dead_code)]
pub struct MPU {
    _marker: PhantomData<*const ()>,
}

impl MPU {
    pub unsafe fn arm() {
        let mpu = MPU.as_ref().unwrap().deref();
        mpu.ctrl.modify(|b| b | 1);
        asm::dsb();
    }

    pub unsafe fn disarm() {
        let mpu = MPU.as_ref().unwrap().deref();
        mpu.ctrl.modify(|b| b & !1);
        asm::dsb();
    }


}
