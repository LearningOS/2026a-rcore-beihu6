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
use alloc::vec::Vec;
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
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// maximum number of syscalls
const MAX_SYSCALL_NUM: usize = 512;

mod fs;
mod process;

use fs::*;
use process::*;
lazy_static! {
    /// Per-task syscall counters, indexed by `task_id * MAX_SYSCALL_NUM + syscall_id`.
    ///
    /// NOTE: keep the storage on the heap. Building a `[_; MAX_APP_NUM]` array
    /// here would first construct it on the (8 KiB) kernel stack and overflow it.
    pub static ref SYSCALL_COUNT: UPSafeCell<Vec<u64>> = unsafe {
        UPSafeCell::new(alloc::vec![0u64; MAX_APP_NUM * MAX_SYSCALL_NUM])
    };
}

/// Index of `syscall_id` for task `task_id` in [`SYSCALL_COUNT`].
fn syscall_count_index(task_id: usize, syscall_id: usize) -> usize {
    task_id * MAX_SYSCALL_NUM + syscall_id
}

/// Get how many times `syscall_id` was invoked by task `task_id`.
pub fn syscall_count(task_id: usize, syscall_id: usize) -> u64 {
    if syscall_id >= MAX_SYSCALL_NUM {
        return 0;
    }
    SYSCALL_COUNT.exclusive_access()[syscall_count_index(task_id, syscall_id)]
}

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    if syscall_id < MAX_SYSCALL_NUM {
        let current_task_id = get_current_task_id();
        SYSCALL_COUNT.exclusive_access()[syscall_count_index(current_task_id, syscall_id)] += 1;
    }
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

