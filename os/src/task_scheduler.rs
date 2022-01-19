use core::marker::Copy;
use cortex_m::interrupt;

#[derive(Copy, Clone)]
pub enum ProcessState {
    Initialize,
    Running,
    Ready,
    Blocked,
    Terminated,
}

#[derive(Copy, Clone)]
pub struct OptionalStruct<T> {
    pub is_some: bool,
    pub value: T,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ProcessControlBlock {
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

pub struct TaskScheduler {
    pcbs: [OptionalStruct<ProcessControlBlock>; 12],
}

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
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
                    // name: [0; 8],
                    // next: None,
                },
            }; 12],
        }
    }

    pub fn tick(&mut self) {

    }
}
