//! 任务间切换演示程序
#![no_std]
#![no_main]
// tornado-user/src/bin/channel.rs-COMMENT: 2022-11-04 Fri Andre :] remote the asm feature
// #![feature(asm)]
// tornado-user/src/bin/channel.rs-COMMENT: 2022-11-04 Fri Andre :] remote the llvm_asm feature
// #![feature(llvm_asm)]

extern crate alloc;
#[macro_use]
extern crate tornado_user;

// tornado-user/src/bin/channel.rs-COMMENT: 2022-11-04 Fri Andre :] import asm macro
use core::arch::asm;

use alloc::sync::Arc;

use tornado_user::{execute_async_main, spawn, task::channel::bounded};
async fn async_main() -> i32 {
    let (tx, rx) = bounded::<u8, 20>();
    spawn(async move {
        let receiver = Arc::new(rx);
        println!("[user] start receive from channel");
        let ret = receiver.receive().await;
        println!("[user] received {} from channel", ret);
    });
    spawn(async move {
        let sender = Arc::new(tx);
        println!("[user] send 0 to channel");
        sender.send(0).await;
    });
    0
}

// 异步main函数，由entry调用execute_async_main
#[no_mangle]
fn main() -> i32 {
    execute_async_main(async_main())
}
