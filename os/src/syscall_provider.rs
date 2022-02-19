use core::{arch::asm, ptr::read_volatile};
use stm32f1xx_hal::{device};
use cortex_m::peripheral::SCB;
use cortex_m_rt::{exception};
use cortex_m_semihosting::hprintln;

use crate::{task_scheduler::{SavedState, self}, TASK_SCHEDULER};

#[allow(unused_macros)]

macro_rules! syscall {
    ($id:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
        // let _ = hprintln!("syscall: {}", stringify!($id));
        {
            let mut return_value: u32;
            unsafe { 
                asm!("
                    svc 0
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
        STMFD SP!, {{R4-R12}}
        MOV R1, SP
        BL {handler}
        LDMFD SP!, {{R4-R12}}
        POP {{PC}}
    ", handler = sym svc_handler, options(noreturn));
}

// svc_args => R0
pub unsafe extern "C" fn svc_handler(caller_stack_addr: * const u32, exc_stack_addr: * const u32) {
    let syscall_id = *caller_stack_addr;
    let arg1 = *caller_stack_addr.offset(1);
    let arg2 = *caller_stack_addr.offset(2);
    let arg3 = *caller_stack_addr.offset(3);

    let pc = *caller_stack_addr.offset(6);

    #[cfg(debug_assertions)]
    hprintln!("SVC #{}", syscall_id).unwrap();


    match syscall_id {
        0 => {
            
        },
        1 => {
            // Yield
            SCB::set_pendsv();
            asm!("
                dsb
                isb
            ");
        },
        _ => {
            panic!("unknown syscall: {}", syscall_id);
        }
    }
}


#[naked]
#[no_mangle]
pub unsafe extern "C" fn PendSV() {
    asm!("
        PUSH {{LR}}
        STMFD SP!, {{R4-R12}}
        TST LR, #4
        ITE EQ
        MRSEQ R0, MSP
        MRSNE R0, PSP
        MOV R1, SP
        MOV R2, LR
        BL {handler}
        LDMFD SP!, {{R4-R12}}
        POP {{PC}}
    ", handler = sym pendsv_handler, options(noreturn));
    // Note that LDMFD might not be executed if a context switch occurs.
}

pub unsafe extern "C" fn pendsv_handler(caller_stack_addr: * const u32, exc_stack_addr: * const u32, exc_return: * const u32) {

    // Check Task Scheduler status

    if TASK_SCHEDULER.is_none() {
        #[cfg(debug_assertions)]
        hprintln!("Task scheduler is not initialized").unwrap();
        return
    }

    let task_scheduler = TASK_SCHEDULER.as_mut().unwrap();

    if !task_scheduler.is_activated {
        // need to initiate the root task
        task_scheduler.init_handler(); // branch away!
    } else {
        task_scheduler.switch();
    }

    // let r4 = *exc_stack_addr.offset(0);
    // let r5 = *exc_stack_addr.offset(1);
    // let r6 = *exc_stack_addr.offset(2);
    // let r7 = *exc_stack_addr.offset(3);
    // let r8 = *exc_stack_addr.offset(4);
    // let r9 = *exc_stack_addr.offset(5);
    // let r10 = *exc_stack_addr.offset(6);
    // let r11 = *exc_stack_addr.offset(7);

    // let saved_state = SavedState {
    //     r4: *exc_stack_addr.offset(0),
    //     r5: *exc_stack_addr.offset(1),
    //     r6: *exc_stack_addr.offset(2),
    //     r7: *exc_stack_addr.offset(3),
    //     r8: *exc_stack_addr.offset(4),
    //     r9: *exc_stack_addr.offset(5),
    //     r10: *exc_stack_addr.offset(6),
    //     r11: *exc_stack_addr.offset(7),
    //     psp: caller_stack_addr as u32,
    //     exc_return: exc_return as u32,
    // };


}

// pub unsafe extern "C" fn PendSV() {
//     asm!("
//         mrs r0, psp
//         ldr r3, #0
//         ldr r2, [r3]
        
//         mrs r1, control
//         stmdb r0!, {{r1, r4-r11}}
//         str r0, [r2]

//         stmdb sp!, {{r3, r14}}
//         mov r0, {max_syscall_interrupt_priority}
//         msr basepri, r0
//         dsb
//         isb
//         bl {task_switch_context}
//         mov r0, #0
//         msr basepri, r0
//         ldmia sp!, {{r3, r14}}

//         ldr r1, [r3]
//         ldr r0, [r1]
//         add r1, r1, #4

//         dmb
//         ldr r2, =0xE000ED94
//         ldr r3, [r2]
//         bic r3, #1
//         str r3, [r2]

//         ldr r2, =0xE000ED9C
//         ldmia r1!, {{r4-r11}}
//         stmia r2!, {{r4-r11}}

//         ldr r2, =0xE000ED94
//         ldr r3, [r2]
//         orr r3, #1
//         str r3, [r2]
//         dsb

//         ldmia r0!, {{r3, r4-r11}}
//         msr control, r3

//         msr psp, r0
//         bx r14
//     ", max_syscall_interrupt_priority = const 0, task_switch_context = const 0, options(noreturn))
// }
