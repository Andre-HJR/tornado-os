//! 任务与进程上下文切换对比性能测试程序，任务部分
#![no_std]
#![no_main]
// tornado-user/src/bin/analysis0.rs-COMMENT: 2022-11-04 Fri Andre :] remove the asm feature
// #![feature(asm)]
// tornado-user/src/bin/analysis0.rs-COMMENT: 2022-11-04 Fri Andre :] remove the llvm_asm feature
// #![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate tornado_user;

// tornado-user/src/bin/analysis0.rs-COMMENT: 2022-11-04 Fri Andre :] import the asm macro, remove

use tornado_user::{do_yield, execute_async, read_timer, reset_timer, spawn};

async fn a(_x: usize) {}

// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    for i in 0..200 {
        spawn(a(i));
    }
    reset_timer();
    execute_async();
    println!("[analysis] coroutines timer: {}", read_timer());
    do_yield(2);
    0
}
