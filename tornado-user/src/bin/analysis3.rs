//! 对比rCore-Tutorial-v3任务切换性能测试
#![no_std]
#![no_main]
// tornado-user/src/bin/analysis3.rs-COMMENT: 2022-11-04 Fri Andre :] remote the asm feature
// #![feature(asm)]
// tornado-user/src/bin/analysis3.rs-COMMENT: 2022-11-04 Fri Andre :] remote the llvm_asm feature
// #![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate tornado_user;

// tornado-user/src/bin/analysis3.rs-COMMENT: 2022-11-04 Fri Andre :] import asm macro
use core::arch::asm;

use tornado_user::{execute_async, read_timer, spawn};

const MAX_TASK: usize = 50;

async fn a(_x: usize) {}

// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    let start = read_timer();
    for i in 0..MAX_TASK {
        spawn(a(i));
    }
    execute_async();
    let end = read_timer();
    println!("[analysis] tornado-os time: {}", end - start);
    0
}
