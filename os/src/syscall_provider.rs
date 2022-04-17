use core::{arch::asm, ptr::read_volatile};
use stm32f1xx_hal::{device};
use cortex_m::{peripheral::SCB, asm::dsb};
use cortex_m_rt::{exception};
// use cortex_m_semihosting::{hprintln, hprint};
use crate::{hprintln, hprint};

use crate::{task_scheduler::{SavedState, self, ProcessState}, TASK_SCHEDULER, usb_hid};

#[allow(unused_macros)]

#[macro_export]
macro_rules! syscall {
    ($id:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
        // let _ = hprintln!("syscall: {}", stringify!($id));
        {
            let mut return_value: u32;
            unsafe { 
                core::arch::asm!("
                    svc 0
                    ISB
                ", 
                    inout("r0") $id => return_value, 
                    inout("r1") $arg1 => _, 
                    inout("r2") $arg2 => _, 
                    inout("r3") $arg3 => _
                ); 
            }
            return_value
        }

    };
}


// ARM EABI Syscall ABI: 
//     r7: syscall #
//     r0 - r6: arguments
//     r0: return value
#[naked]
#[no_mangle]
pub unsafe extern "C" fn SVCall() {
    asm!("
        TST LR, #4
        ITE EQ
        MRSEQ R0, MSP
        MRSNE R0, PSP
        PUSH {{LR}}
        MOV R1, SP
        BL {handler}
        POP {{PC}}
    ", handler = sym svc_handler, options(noreturn));
}

// R0 => caller_stack_addr
// R1 => exc_stack_addr
// retn => R0
pub unsafe extern "C" fn svc_handler(caller_stack_addr: * const u32, exc_stack_addr: * const u32) -> u32 {

    // Basic Frame:
    // R0, R1, R2, R3, R12, LR, PC, xPSR

    let syscall_id = *caller_stack_addr;
    let arg1 = *caller_stack_addr.offset(1);
    let arg2 = *caller_stack_addr.offset(2);
    let arg3 = *caller_stack_addr.offset(3);

    let pc = *caller_stack_addr.offset(6);

    let _ = hprintln!("[Exception] SVCall: System Call {} ({:#x}, {:#x}, {:#x})", syscall_id, arg1, arg2, arg3);
    // let _ = usb_hid::send_msg(5);

    match syscall_id {
        0 => {
            // Reserved
            
        },
        1 => {
            // Yield
            SCB::set_pendsv();
            dsb();
        },
        3 => {
            // print
            let text = *(arg1 as * const &str) as &str;
            let _ = hprint!("{}", text);
        },
        4 => {
            // C compatible print
            let text = cstr_core::CStr::from_ptr(arg1 as * const u8);
            let _ = hprint!("{}", text.to_str().unwrap());
        },
        5 => {
            // _exit
            let current_pid = TASK_SCHEDULER.as_ref().unwrap().current_process;
            let return_code = arg1;
            TASK_SCHEDULER.as_mut().unwrap().exit(current_pid as u16);
            let _ = hprintln!("process {} exited, return code {}");
            SCB::set_pendsv();
            dsb();
        }, 6 => {
            // create
            let address = arg1;
            let task_scheduler = TASK_SCHEDULER.as_mut().unwrap();
            let current_pid = task_scheduler.current_process;
            let pid = task_scheduler.create(current_pid, address).unwrap().pid;
            task_scheduler.set_pending_process(pid);
            // jump to 
            SCB::set_pendsv();
            dsb();
        }
        _ => {
            panic!("unknown syscall: {}", syscall_id);
        }
    }

    0
}


#[naked]
#[no_mangle]
pub unsafe extern "C" fn PendSV() {
    asm!("
        PUSH {{LR}}
        TST LR, #4
        ITE EQ
        MRSEQ R0, MSP
        MRSNE R0, PSP
        STMDB R0!, {{R4-R11}}
        MOV R1, SP
        MOV R2, LR
        MOV R3, R7
        BL {handler}
        POP {{LR}}
        TST LR, #4
        ITE EQ
        MRSEQ R0, MSP
        MRSNE R0, PSP
        SUB R0, #32
        LDMIA R0!, {{R4-R11}}
        BX LR
    ", handler = sym pendsv_handler, options(noreturn));
    // Note that LDMFD might not be executed if a context switch occurs.
}

pub unsafe extern "C" fn pendsv_handler(caller_stack_addr: * const u32, exc_stack_addr: * const u32, exc_return: * const u32, stack_base: * const u32) {

    let _ = hprintln!("[Context Switch] PendSV - PSP: {:#x} SP: {:#x} LR: {:#x} R7: {:#x}", caller_stack_addr as u32, exc_stack_addr as u32, exc_return as u32, stack_base as u32);

    // Check Task Scheduler status

    if TASK_SCHEDULER.is_none() {
        #[cfg(debug_assertions)]
        let _ = hprintln!("Task scheduler is not initialized");
        return
    }

    // let _ = usb_hid::send_msg(7);

    let saved_state = SavedState {
        rsp: exc_stack_addr as u32,
        psp: caller_stack_addr as u32,
        exc_return: exc_return as u32,
    };

    let task_scheduler = TASK_SCHEDULER.as_mut().unwrap();

    if !task_scheduler.is_activated {
        let _ = hprintln!("[Context Switch] PendSV - Init Root Process");
        // need to initiate the root task
        task_scheduler.init_handler(saved_state); 

    } else {
        let new_process_block = task_scheduler.switch(saved_state);
        if new_process_block.state == ProcessState::Initialize {
            let _ = hprintln!("[Context Switch] PendSV - Initialize process {}", new_process_block.pid);
            // require initialization
            asm!("
                MOV R5, {SP}
                STMDB R5!, {{{ZERO}}}
                STMDB R5!, {{{PC}}}
                STMDB R5!, {{{ZERO}}}
                STMDB R5!, {{{ZERO}}}
                STMDB R5!, {{{ZERO}}}
                STMDB R5!, {{{ZERO}}}
                STMDB R5!, {{{ZERO}}}
                STMDB R5!, {{{ZERO}}}
                MSR PSP, R5
                MOV PC, {EXC_RETURN}
            ",
                ZERO = in(reg) 0,
                PC = in(reg) new_process_block.entry_point,
                SP = in(reg) new_process_block.stack_base,
                EXC_RETURN = in(reg) 0xFFFF_FFFDu32,
                in("r5") 0,
                options(noreturn)
            );
        } else {
            let _ = hprintln!("[Context Switch] PendSV - Serializing and switching to {}", new_process_block.pid);
            let load_state = new_process_block.running_state;
            asm!("
                MOV R0, {psp}
                LDMIA R0!, {{R4-R11}}
                MSR PSP, R0
                MOV PC, {exc_return}
            ", 
                // rsp = in(reg) load_state.rsp,
                psp = in(reg) load_state.psp,
                exc_return = in(reg) load_state.exc_return,
                in("r0") 0,
                options(noreturn)
            );
        }
    }
}

