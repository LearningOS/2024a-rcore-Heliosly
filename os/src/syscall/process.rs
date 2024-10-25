//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::{CLOCK_FREQ, MAX_SYSCALL_NUM, PAGE_SIZE},
    loader::get_app_data_by_name,
    mm::{translated_byte_buffer, translated_refmut, translated_str, MapPermission, VirtAddr},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next, suspend_current_and_run_next, TaskControlBlock, TaskStatus
    }, timer::{get_time, get_time_us},
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
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel:pid[{}] sys_yield", current_task().unwrap().pid.0);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    //let task = current_task().unwrap();
    // find a child process
    

    
    // ---- access current PCB exclusively
    //let mut inner = task.inner_exclusive_access();
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
    /*  if inner.isspaw{
      let pid11:usize=task.getpid();
      if inner.is_zombie(){
         
        let exit_code =inner.exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
       
        pid11 as isize
      }
      else{
         
        -2
      }
         
    }
    else                                          {
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically

                                 } */
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_get_time NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
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
    trace!(
        "kernel:pid[{}] sys_task_info NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    -1
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_mmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );if _start % PAGE_SIZE != 0 || _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;  
    }
    let arc=current_task().unwrap();
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
    if  arc.inner_exclusive_access().memory_set.insert_framed_area_peek(start, end, flag)
    {-1}
    else{
        0
    }

}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_munmap NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _start % PAGE_SIZE != 0  {
        return -1;  
    }
    let _end=_start+_len;
    let end:VirtAddr=_end.into();
    let start:VirtAddr=_start.into();
    let arc=current_task().unwrap();
    
    if arc.inner_exclusive_access().memory_set.unmap_peek(start, end){
        return -1
    }
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let current_task = current_task().unwrap();
    let token =current_user_token();
    let path = translated_str(token, _path);
    let tcb=Arc::new(TaskControlBlock::new(get_app_data_by_name(path.as_str()).unwrap()));
    let mut inner=tcb.inner_exclusive_access();
    let mut pin=current_task.inner_exclusive_access();
    inner.parent=Some(Arc::downgrade(&current_task));
    pin.children.push(tcb.clone());
    drop(inner);
    let pid = tcb.pid.0 as isize;
    add_task(tcb);
    pid
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    if _prio>=2{
        _prio
    }
    else {
        -1
    }
}
