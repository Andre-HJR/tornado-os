//! 任务切换演示程序1
#![no_std]
#![no_main]
// tornado-user/src/bin/yield-task1.rs-COMMENT: 2022-11-04 Fri Andre :] remove the asm feature
// #![feature(asm)]
// tornado-user/src/bin/yield-task1.rs-COMMENT: 2022-11-04 Fri Andre :] remove the llvm_asm feature
// #![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate tornado_user;

// tornado-user/src/bin/yield-task1.rs-COMMENT: 2022-11-04 Fri Andre :] import asm macro, remvoe

use tornado_user::execute_async_main;
async fn async_main() -> i32 {
    println!("[user] yield test task 1");
    0
}

// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    execute_async_main(async_main())
}
