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


#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            extern "C" {
                static mut _sbss: u8;
                static mut _ebss: u8;

                static mut _sdata: u8;
                static mut _edata: u8;
                static _sidata: u8;
            }

            let count = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
            core::ptr::write_bytes(&mut _sbss as *mut u8, 0, count);

            let count = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
            core::ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count);

            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    }
}
