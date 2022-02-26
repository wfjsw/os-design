use core::marker::PhantomData;
use cortex_m::asm;
use cortex_m::register::control;

pub struct Npriv {
    _marker: PhantomData<*const ()>,
}

impl Npriv {
    pub unsafe fn is_privileged() -> bool {
        let ctrl = control::read();
        ctrl.npriv() == cortex_m::register::control::Npriv::Privileged
    }

    pub unsafe fn set_privileged() {
        let mut ctrl = control::read();
        ctrl.set_npriv(cortex_m::register::control::Npriv::Privileged);
        control::write(ctrl);
        asm::dsb();
    }

    pub unsafe fn set_unprivileged() {
        let mut ctrl = control::read();
        ctrl.set_npriv(cortex_m::register::control::Npriv::Unprivileged);
        control::write(ctrl);
        asm::dsb();
    }
}
