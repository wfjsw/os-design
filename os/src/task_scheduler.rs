use core::marker::Copy;
use cortex_m::{interrupt, peripheral::SCB};

use crate::structs::OptionalStruct;



#[derive(Copy, Clone, PartialEq)]
pub enum ProcessState {
    Initialize,
    Running,
    Ready,
    Blocked,
    Terminated,
}



#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ProcessControlBlock {
    // Process ID
    pub pid: u16, 
    pub ppid: u16,
    pub stack_pointer: u32,
    pub stack_base: u32,
    pub stack_size: u32,
    pub priority: u8,
    pub state: ProcessState,
    // name: [u8; 8],
    // next: Option<&'static mut ProcessControlBlock>,
}

// Max processes. This is mainly limited by the memory available.
pub const MAX_PCB : usize = 8;

#[repr(C)]
pub struct TaskScheduler {
    pub is_activated: bool,
    pub current_process: usize,
    pub pcbs: [OptionalStruct<ProcessControlBlock>; MAX_PCB],
}

/// ARMvx-M volatile registers that must be saved across context switches.
#[repr(C)]
#[derive(Debug, Default)]
pub struct SavedState {
    // NOTE: the following fields must be kept contiguous!
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub psp: u32,
    pub exc_return: u32, // effectively pc
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct BaseExceptionFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: u32,
    xpsr: u32,
}

// Memory management:
// All process shall equally allocate 

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            is_activated: false,
            current_process: 0,
            pcbs: [OptionalStruct {
                is_some: false,
                value: ProcessControlBlock {
                    pid: 0,
                    ppid: 0,
                    stack_pointer: 0,
                    stack_base: 0,
                    stack_size: 0,
                    priority: 0,
                    state: ProcessState::Initialize,
                },
            }; MAX_PCB],
        }
    }

    pub fn init(&mut self) {
        if self.pcbs[0].is_some {
            panic!("task scheduler is in an inconsistent state - pid 0 is already in use");
        }
        self.pcbs[0].is_some = true;
        self.pcbs[0].value.pid = 0;
        self.pcbs[0].value.ppid = 0;
        self.pcbs[0].value.state = ProcessState::Initialize;
        self.pcbs[0].value.priority = 0;

        // Trigger context switch
        SCB::set_pendsv();
    }

    pub fn init_handler(&mut self) {
        // intended to call in handler mode
        self.is_activated = true;
        self.pcbs[0].is_some = true;
        self.pcbs[0].value.state = ProcessState::Running;
        self.current_process = 0;

        // init MPU and prepare to drop into thread mode
    }

    pub fn create(&mut self, ppid: u16) -> Option<&mut ProcessControlBlock> {
        let mut i = 0;
        while i < 12 {
            if self.pcbs[i].is_some {
                i += 1;
            } else {
                self.pcbs[i].is_some = true;
                self.pcbs[i].value.ppid = ppid;
                self.pcbs[i].value.pid = i as u16;
                self.pcbs[i].value.state = ProcessState::Initialize;
                self.pcbs[i].value.stack_base = get_base_stack_pointer_from_pid(i);
                return Some(&mut self.pcbs[i].value)
            }
        }

        None
    }

    // pub fn next(&mut self) -> Option<&ProcessControlBlock> {
    //     let mut i = self.current_process;
    //     while i < 12 {
    //         if self.pcbs[i].is_some {
    //             self.current_process = i;
    //             break;
    //         }
    //         i += 1;

    //         if i > 11 {
    //             i = 0;
    //         }

    //         if i == self.current_process {
    //             return None
    //         }
    //     }

    //     Some(&self.pcbs[i].value)
    // }

    pub fn nextReady(&mut self) -> Option<&ProcessControlBlock> {
        let mut i = self.current_process;
        while i < 12 {
            if self.pcbs[i].is_some && self.pcbs[i].value.state == ProcessState::Ready {
                self.current_process = i;
                break;
            }
            i += 1;

            if i > 11 {
                i = 1;
            }

            if i == self.current_process {
                return None
            }
        }

        Some(&self.pcbs[i].value)
    }

    // this simulates a process that is executed when processor is idle
    // we try not to schedule to this process
    pub fn idle() -> ! {
        loop {}
    } 

    pub fn switch(&mut self) {
        let next_process = self.nextReady();
        if next_process.is_none() {
            // switch to idle process
        }
    }

}

// https://crates.io/crates/thumb2-stack-size
// hardcoded base sp 
// the division line between stack and heap lays in 0x20008000
// 0x2000F500 - 0x2000FFFF is for bootloader flags (reserved)
// OS occupies 0x2000E000 - 0x2000F500
// Each process occupy 0xC00 (3072 bytes) of stack
fn get_base_stack_pointer_from_pid(pid: usize) -> u32 {
    0x2000E000 - (pid as u32) * 0xC00
}
