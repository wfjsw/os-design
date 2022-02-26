use core::arch::asm;

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
pub fn print(str: &str) {
    syscall!(3, &str as * const &str as u32, 0, 0);
}

pub fn println(str: &str) {
    print(str);
    print("\n");
}

macro_rules! printf {
    ($fmt:expr) => {
        print($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        print(&format!($fmt, $($arg)*));
    };
}
