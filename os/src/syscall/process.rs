//! Process management syscalls




//use core::mem;




use core::mem;

use crate::{
    config::{CLOCK_FREQ, MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translated_byte_buffer, MapPermission, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER
    }, timer::{get_time, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}
bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct Portpomiss: u8 {
        ///Readable
        const R = 1 << 0;
        ///Writable
        const W = 1 << 1;
        ///Excutable
        const X = 1 << 2;
        
    }
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
    
    let usec=get_time_us().to_ne_bytes();
   
    let sec =(get_time()/CLOCK_FREQ).to_ne_bytes();
   
   
    let _cur=current_user_token();
    let bufs=translated_byte_buffer(_cur, _ts as *const u8, 16);
    let mut i=0;
    for buf in bufs{
        for atm in buf{
            if i>=8{
                *atm = usec[i-8];
                }
                else{
                *atm=sec[i];
                }
        
                i+=1;
        }
    }
    0
    
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
     let (a,b)=TASK_MANAGER.readinfo();
    let len = mem::size_of::<TaskInfo>();
    let bufs = translated_byte_buffer(current_user_token(),_ti as *const u8, len);
    let mut scr=[0u32;500];
    for _b in  b{
        scr[_b.0]=_b.1;
    }
    let info=TaskInfo {
     status:TaskStatus::Running,
     syscall_times:scr,
     time:a,
    };

       
    let u8_info: &[u8] = unsafe {
        core::slice::from_raw_parts(&info as *const _ as *const u8, mem::size_of::<TaskInfo>())
    };
    let mut i=0;
    for buf in bufs{
        for atm in buf{
            *atm =  u8_info[i];
            i+=1;
        }


    }
    
    
       
        
    
    
    


    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 || _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;  
    }
    
    let portpomis = Portpomiss::from_bits_truncate(_port as u8);
    let mut flag:MapPermission=MapPermission::empty();
    flag|=MapPermission::U;
    if portpomis.contains(Portpomiss::R){
         flag|=MapPermission::R;
    }
    if portpomis.contains(Portpomiss::W){
         flag|=MapPermission::W;
    }
    if portpomis.contains(Portpomiss::X){
        flag|=MapPermission::X;
    }
    let _end=_start+_len;
   
    let end:VirtAddr=_end.into();
    let start:VirtAddr=_start.into();
     


    if TASK_MANAGER.loc(start,end,flag){
        return -1;
    }

    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0  {
        return -1;  
    }
    let _end=_start+_len;
    let end:VirtAddr=_end.into();
    let start:VirtAddr=_start.into();
    if TASK_MANAGER.unloc(start,end){
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
