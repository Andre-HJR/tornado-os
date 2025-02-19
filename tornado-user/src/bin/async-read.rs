//! 异步IO系统调用演示程序
#![no_std]
#![no_main]
// tornado-user/src/bin/async-read.rs-COMMENT: 2022-11-04 Fri Andre :] remove the asm feature
// #![feature(asm)]
// tornado-user/src/bin/async-read.rs-COMMENT: 2022-11-04 Fri Andre :] remove the llvm_asm feature
// #![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate tornado_user;

// tornado-user/src/bin/async-read.rs-COMMENT: 2022-11-04 Fri Andre :] import asm macro, remove

use tornado_user::{execute_async_main, io::read_block};
async fn async_main() -> i32 {
    let mut buf = [0; 512];
    read_block(0, &mut buf).await;
    println!("[user] async read block ret: {:x?}", &buf[0..10]);
    0
}

// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    execute_async_main(async_main())
}
