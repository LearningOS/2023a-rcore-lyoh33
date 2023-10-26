//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, into_pa, curtask_runtime, curtask_syscall_times, curtask_any_mapped, curtask_any_unmapped, insert_curtask_framed_area, remove_curtask_framed_area,
    }, timer::{get_time_us}, mm::{VirtAddr, MapPermission},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    //_ts指向用户空间的虚拟地址，现在处于内核空间
    //需要查页表将_ts转换转换成原来对应的物理地址
    let ts = into_pa(_ts as *const u8) as *mut TimeVal;
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let ti = into_pa(_ti as *const u8) as *mut TaskInfo;
    let status = TaskStatus::Running;
    let time = curtask_runtime();
    let syscall_times = curtask_syscall_times();
    unsafe{
        *ti = TaskInfo{
            status,
            time,
            syscall_times
        };
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if !VirtAddr(_start).aligned(){
        return -1;
    }
    if _port & !0x7 != 0 || _port & 0x7 == 0{
        return -1;
    }
    let start_va = VirtAddr(_start);
    let end_va = VirtAddr(_start + _len).ceil().into();
    let mut permission = MapPermission::U;
    if _port & 0b1 != 0 {
        permission |= MapPermission::R;
    }
    if _port & 0b10 != 0{
        permission |= MapPermission::W;
    }
    if _port & 0b100 != 0{
        permission |= MapPermission::X;
    }
    if curtask_any_mapped(start_va, end_va){
        return -1;
    }
    insert_curtask_framed_area(start_va, end_va, permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    // trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if !VirtAddr(_start).aligned(){
        return -1;
    }
    let start_va = VirtAddr(_start).floor().into();
    let end_va = VirtAddr(_start + _len).ceil().into();
    if curtask_any_unmapped(start_va, end_va){
        return -1;
    }
    remove_curtask_framed_area(start_va, end_va);
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
