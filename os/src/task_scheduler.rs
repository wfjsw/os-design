use core::marker::Copy;
use core::ops::Deref;
use cortex_m::{interrupt, peripheral::SCB, register::control};
// use cortex_m_semihosting::hprintln;
use crate::hprintln;

use crate::{
    structs::OptionalStruct,
    utils::{mpu::MPU, npriv::Npriv}, syscall,
};

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
    pub pid: usize,
    pub ppid: usize,
    pub stack_base: u32,
    pub entry_point: u32,
    pub priority: u8,
    pub state: ProcessState,
    pub running_state: SavedState,
}

// Max processes. This is mainly limited by the memory available.
pub const MAX_PCB: usize = 8;

#[repr(C)]
pub struct TaskScheduler {
    pub is_activated: bool,
    pub current_process: usize,
    pub pending_process: usize,
    pub pcbs: [OptionalStruct<ProcessControlBlock>; MAX_PCB],
}

/// ARMvx-M volatile registers that must be saved across context switches.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct SavedState {
    // pub r4: u32,
    // pub r5: u32,
    // pub r6: u32,
    // pub r7: u32,
    // pub r8: u32,
    // pub r9: u32,
    // pub r10: u32,
    // pub r11: u32,
    pub rsp: u32,        // stack pointer saving register value
    pub psp: u32,        // stack pointer
    pub exc_return: u32, // effectively pc
}

// Memory management:
// All process shall equally allocate

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            is_activated: false,
            current_process: 0,
            pending_process: 0,
            pcbs: [OptionalStruct {
                is_some: false,
                value: ProcessControlBlock {
                    pid: 0,
                    ppid: 0,
                    stack_base: 0,
                    entry_point: 0,
                    priority: 0,
                    state: ProcessState::Initialize,
                    running_state: SavedState {
                        rsp: 0,
                        psp: 0,
                        exc_return: 0,
                    },
                },
            }; MAX_PCB],
        }
    }

    pub fn init(&mut self) -> ! {
        if self.pcbs[0].is_some() {
            panic!("task scheduler is in an inconsistent state - pid 0 is already in use");
        }
        self.pcbs[0].is_some = true;
        self.pcbs[0].value.pid = 0;
        self.pcbs[0].value.ppid = 0;
        self.pcbs[0].value.state = ProcessState::Initialize;
        self.pcbs[0].value.priority = 0;

        self.pcbs[0].value.stack_base = get_base_stack_pointer_from_pid(0);

        // get address of sub_main
        self.pcbs[0].value.entry_point = crate::sub_main as *const u32 as u32;

        unsafe {
            let _ = hprintln!("[Task Scheduler] Init: Duplicate PSP to {:#x}", self.pcbs[0].value.stack_base);
            core::arch::asm!("
                MSR PSP, {SP}
                MRS R0, CONTROL 
                ORRS R0, R0, #0x2 
                MSR CONTROL, R0
                ISB
            ", SP = in(reg) self.pcbs[0].value.stack_base,
            out("r0") _);

            // let _ = hprintln!("[Task Scheduler] Init: Set SPsel to PSP");
            // let mut ctrl = control::read();
            // ctrl.set_spsel(control::Spsel::Psp);
            // control::write(ctrl);

            // Npriv::set_unprivileged();

        }

       

        let _ = hprintln!("[Task Scheduler] Init: Set PendSV");
        // Trigger context switch
        // SCB::set_pendsv();
        syscall!(1, 0, 0, 0);

        loop {}

        // let _ = hprintln!("[Task Scheduler] Init: Waiting for interruption");

        // cortex_m::asm::wfi();
    }

    pub unsafe fn init_handler(&mut self, state: SavedState) -> u32 {
        // intended to call in handler mode

        self.pcbs[0].is_some = true;
        let this_pcb = self.pcbs[0].unwrap();
        this_pcb.state = ProcessState::Running;
        this_pcb.running_state = state;
        // this_pcb.stack_base = stack_base;
        this_pcb.running_state.psp = this_pcb.stack_base;
        self.current_process = 0;
        self.is_activated = true;


        this_pcb.entry_point
        // init MPU and prepare to drop into thread mode
        // Npriv::set_unprivileged();
        // MPU::arm();
    }

    pub fn create(&mut self, ppid: usize, entry_point: u32) -> Option<&ProcessControlBlock> {
        let mut i = 1;
        while i < 12 {
            if self.pcbs[i].is_some() {
                i += 1;
            } else {
                self.pcbs[i].is_some = true;
                self.pcbs[i].value.ppid = ppid;
                self.pcbs[i].value.pid = i;
                self.pcbs[i].value.state = ProcessState::Initialize;
                self.pcbs[i].value.stack_base = get_base_stack_pointer_from_pid(i);
                self.pcbs[i].value.entry_point = entry_point;

                let _ = hprintln!("[Task Scheduler] Process {} created, ppid {}", i, ppid);

                return Some(&self.pcbs[i].value);
            }
        }

        None
    }

    pub fn next_ready(&mut self) -> &mut ProcessControlBlock {
        let mut i = self.current_process;
        while i < 12 {
            if self.pcbs[i].is_some() && self.pcbs[i].value.state == ProcessState::Ready {
                self.current_process = i;
                break;
            }
            
            i += 1;

            if i > 11 {
                i = 1;
            }

            if i == self.current_process {
                self.current_process = 0;
                return &mut self.pcbs[0].value;
            }
        }

        &mut self.pcbs[i].value
    }

    pub fn this_process_status(&self) -> ProcessState{
        self.pcbs[self.current_process].value.state
    }

    pub fn switch(&mut self, old_saved_state: SavedState) -> &ProcessControlBlock {
        // disarm MPU first
        // MPU::disarm();

        let this_process = &mut self.pcbs[self.current_process].value;

        this_process.state = ProcessState::Ready;
        this_process.running_state = old_saved_state;

        let next_process = match self.pending_process {
            0 => {
                let process = self.next_ready();
                process.state = ProcessState::Running;
                process
            },
            _ => {
                let pending_process = self.pending_process;
                self.pending_process = 0;
                &mut self.pcbs[pending_process].value
            },
        };

        // setup MPU
        // TBD

        // arm MPU
        // MPU::arm();

        next_process
    }

    pub fn set_pending_process(&mut self, pid: usize) {
        if pid > 0 && pid < MAX_PCB {
            self.pending_process = pid;
        }
    }


    pub fn exit(&mut self, pid: u16) {
        self.pcbs[pid as usize].value.state = ProcessState::Terminated;
        self.pcbs[pid as usize].is_some = false;
    }
}

// https://crates.io/crates/thumb2-stack-size
// hardcoded base sp
// the division line between stack and heap lays in 0x20008000
// 0x2000F500 - 0x2000FFFF is for bootloader flags (reserved)
// OS occupies 0x2000E000 - 0x2000F500
// Each process occupy 0xC00 (3072 bytes) of stack
fn get_base_stack_pointer_from_pid(pid: usize) -> u32 {
    0x2000D000 - (pid as u32) * 0x1000
}
