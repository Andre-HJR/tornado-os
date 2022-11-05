//! 任务与进程上下文切换对比性能测试程序，进程第二部分
#![no_std]
#![no_main]
// tornado-user/src/bin/analysis2.rs-COMMENT: 2022-11-04 Fri Andre :] remove the asm feature
// #![feature(asm)]
// tornado-user/src/bin/analysis2.rs-COMMENT: 2022-11-04 Fri Andre :] remove the llvm_asm feature
// #![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate tornado_user;

// tornado-user/src/bin/analysis2.rs-COMMENT: 2022-11-04 Fri Andre :] import asm macro, remove

use tornado_user::{do_yield, execute_async_analysis, read_timer, spawn};
async fn analysis_task(_n: usize) {}

// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    for i in 0..100 {
        spawn(analysis_task(i));
        do_yield(2);
    }
    execute_async_analysis();
    println!("[analysis] process timer: {}", read_timer());
    0
}
