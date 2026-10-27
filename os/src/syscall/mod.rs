//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
use crate::sync::UPSafeCell;
use crate::config::MAX_APP_NUM;
use crate::task::get_current_task_id;
use lazy_static::lazy_static;

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// trace syscall
const SYSCALL_TRACE: usize = 410;
/// maximum number of syscalls
const MAX_SYSCALL_NUM: usize = 512;

mod fs;
mod process;

use fs::*;
use process::*;
/// SyscallCount struct to keep track of syscall counts
pub struct SyscallCount {
    count: [u64; MAX_SYSCALL_NUM]
}
impl SyscallCount {
    /// Create a new instance of SyscallCount
    fn new() -> Self {
        SyscallCount {
            count: [0; MAX_SYSCALL_NUM],
        }
    }
    /// Increment the count for a specific syscall ID
    fn increment(&mut self, syscall_id: usize) {
        if syscall_id < MAX_SYSCALL_NUM {
            self.count[syscall_id] += 1;
        }
    }
    /// Get the count for a specific syscall ID
    pub fn get_count(&self, syscall_id: usize) -> u64 {
        if syscall_id < MAX_SYSCALL_NUM {
            self.count[syscall_id]
        } else {
            0
        }
    }
}

lazy_static! {
    /// Global variable: SYSCALL_COUNT
    pub static ref SYSCALL_COUNT: [UPSafeCell<SyscallCount>; MAX_APP_NUM] = {
        let arr = [(); MAX_APP_NUM].map(|_| unsafe {UPSafeCell::new(SyscallCount::new())});
        arr
    };
}

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    if syscall_id< MAX_SYSCALL_NUM {
        let current_task_id = get_current_task_id();
        SYSCALL_COUNT[current_task_id].exclusive_access().increment(syscall_id);
    }
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
