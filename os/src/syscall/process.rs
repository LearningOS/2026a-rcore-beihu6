//! Process management syscalls
use crate::config::PAGE_SIZE_BITS;
use crate::task::{
    change_program_brk, current_map_anonymous, current_unmap_anonymous, current_user_token,
    exit_current_and_run_next, get_current_task_id, suspend_current_and_run_next,
};
use crate::mm::{translated_byte_buffer, PageTable, PTEFlags, VirtAddr};
use crate::timer::get_time_us;
use crate::syscall::syscall_count;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

fn check_user_access(start: usize, len: usize, readable: bool, writable: bool) -> bool {
    if len == 0 {
        return true;
    }
    let end = match start.checked_add(len) {
        Some(end) => end,
        None => return false,
    };
    let page_table = PageTable::from_token(current_user_token());
    let mut cur = start;
    while cur < end {
        // NOTE: `cur` is a virtual address, so convert it to a page number via
        // `VirtAddr::floor()`; `cur.into()` would truncate it as a VPN instead.
        let pte = match page_table.translate(VirtAddr::from(cur).floor()) {
            Some(pte) => pte,
            None => return false,
        };
        if !pte.is_valid() || !pte.flags().contains(PTEFlags::U) {
            return false;
        }
        if readable && !pte.readable() {
            return false;
        }
        if writable && !pte.writable() {
            return false;
        }
        cur = ((cur >> PAGE_SIZE_BITS) + 1) << PAGE_SIZE_BITS;
        if cur > end {
            cur = end;
        }
    }
    true
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    if !check_user_access(ts as usize, core::mem::size_of::<TimeVal>(), true, true) {
        return -1;
    }
    let time = get_time_us();
    let timeval = TimeVal {
        sec: time / 1_000_000,
        usec: time % 1_000_000,
    };
    let bytes = unsafe {
        core::slice::from_raw_parts(
            (&timeval as *const TimeVal) as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };
    let mut dst = translated_byte_buffer(current_user_token(), ts as *const u8, bytes.len());
    let mut pos = 0;
    for chunk in dst.iter_mut() {
        let n = (bytes.len() - pos).min(chunk.len());
        chunk[..n].copy_from_slice(&bytes[pos..pos + n]);
        pos += n;
        if pos == bytes.len() {
            break;
        }
    }
    0
}

/// Trace read/write/syscall count in user space with page-table permission checks.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            if !check_user_access(_id, 1, true, false) {
                return -1;
            }
            let buf = translated_byte_buffer(current_user_token(), _id as *const u8, 1);
            buf[0][0] as isize
        }
        1 => {
            if !check_user_access(_id, 1, false, true) {
                return -1;
            }
            let mut buf = translated_byte_buffer(current_user_token(), _id as *const u8, 1);
            buf[0][0] = _data as u8;
            0
        }
        2 => {
            let current_task_id = get_current_task_id();
            syscall_count(current_task_id, _id) as isize
        }
        _ => -1,
    }
}

/// Map a page-aligned anonymous memory region into the current user address space.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % 4096 != 0 {
        return -1;
    }
    if prot & !0x7 != 0 || prot == 0 || prot & 0x7 == 0 {
        return -1;
    }
    if start.checked_add(len).is_none() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    if !current_map_anonymous(start, len, prot) {
        return -1;
    }
    0
}

/// Unmap a page-aligned anonymous memory region from the current user address space.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % 4096 != 0 || start.checked_add(len).is_none() {
        return -1;
    }
    if len == 0 {
        return 0;
    }
    if !current_unmap_anonymous(start, len) {
        return -1;
    }
    0
}
    
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
