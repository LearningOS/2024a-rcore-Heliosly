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

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// taskinfo syscall
const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;


use fs::*;
use process::*;

use crate::{task::{TaskStatus, TASK_MANAGER}, timer::get_time_ms};



/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    let mut runid=0;
    {
        let a=TASK_MANAGER.inner.exclusive_access().tasks;
        for (i,task) in a.iter().enumerate(){
            if task.task_status==TaskStatus::Running{
                runid=i;
                break;
            }
        }
    }
    {
        let mut _a=&mut TASK_MANAGER.st.exclusive_access()[runid];
        if *_a==0{
           *_a=get_time_ms();
        }
    }
    {
        let a = &mut TASK_MANAGER.list.exclusive_access()[runid];
        for (sys,_cnt) in a {
            if *sys==syscall_id{
               *_cnt+=1;
               if runid==2{
               //println!("cnt=={},sys={}",_cnt,syscall_id);
               }
            }
        }
    }






    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TASK_INFO => sys_task_info(args[0] as *mut TaskInfo),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
