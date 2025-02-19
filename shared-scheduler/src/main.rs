//! 为飓风内核设计的共享调度器
//! 以二进制包的形式编译
//!
//! 使用方法：将本项目编译出的二进制文件直接烧录到内存中，
//! 然后在内核和用户代码中以基地址实例化共享调度器。
//!
//! 基地址在链接脚本`src/linker-xxx.ld`中指定。同时在烧写的时候也需要指定。
//!
//! 实例化方法：请参考`tornado-kernel/src/task/shared.rs`
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
// shared-scheduler/src/main.rs-COMMENT: 2022-11-04 Fri Andre :] remove the llvm_asm feature
// #![feature(llvm_asm)]
// shared-scheduler/src/main.rs-COMMENT: 2022-11-04 Fri Andre :] remove the asm feature
// #![feature(asm)]
#![feature(maybe_uninit_uninit_array)]
#![feature(naked_functions)]
// shared-scheduler/src/main.rs-COMMENT: 2022-11-04 Fri Andre :] add the asm macro, remove
// shared-scheduler/src/main.rs-COMMENT: 2022-11-06 Sun Andre :] add the const_maybe_uninit_uninit_array
#![feature(const_maybe_uninit_uninit_array)]

extern crate alloc;

#[macro_use]
mod console;
mod algorithm;
mod mm;
mod syscall;
mod task;

use crate::{
    mm::AddressSpaceId,
    task::{
        shared_add_task, shared_delete_task, shared_peek_task, shared_set_task_state,
        SharedScheduler, TaskRepr, TaskResult, TaskState, SHARED_SCHEDULER,
    },
};
use buddy_system_allocator::LockedHeap;
use core::{mem::MaybeUninit, ptr::NonNull};

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 128 * 1024;
static HEAP_MEMORY: MaybeUninit<[u8; HEAP_SIZE]> = core::mem::MaybeUninit::uninit();

#[cfg_attr(not(test), panic_handler)]
pub fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message().unwrap().as_str();
    if let Some(location) = panic_info.location() {
        syscall::sys_panic(
            Some(location.file()),
            location.line(),
            location.column(),
            err,
        );
    } else {
        syscall::sys_panic(None, 0, 0, err);
    }
    // println!("[shared scheduler] panic: {:?}", panic_info);
    unreachable!()
}

// todo: 未来尽量使用有Allocator的new_in函数，这样能处理内存不足的问题
#[cfg_attr(not(test), alloc_error_handler)]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    println!("[shared scheduler] alloc error, layout = {:?}", layout);
    panic!("shared scheduler alloc error: {:?}", layout)
}

/// 共享调度器虚函数表
#[link_section = ".meta"] // 虚函数表只读
#[no_mangle]
pub static SHARED_RAW_TABLE: (
    &'static u8,                        // 共享调度器编译时的基地址
    unsafe extern "C" fn() -> PageList, // 初始化函数，执行完之后，内核将函数指针置空
    &'static SharedScheduler,           // 共享调度器的地址
    unsafe extern "C" fn(NonNull<()>, usize, AddressSpaceId, TaskRepr) -> bool, // 添加任务
    unsafe extern "C" fn(NonNull<()>, extern "C" fn(AddressSpaceId) -> bool) -> TaskResult, // 弹出任务引用
    unsafe extern "C" fn(NonNull<()>, TaskRepr) -> bool, // 删除任务
    unsafe extern "C" fn(NonNull<()>, TaskRepr, TaskState), // 改变任务的状态
) = (
    unsafe { &payload_compiled_start },
    init_payload_environment,
    &SHARED_SCHEDULER,
    shared_add_task,
    shared_peek_task,
    shared_delete_task,
    shared_set_task_state,
);

#[allow(non_upper_case_globals)]
extern "C" {
    // 共享调度器编译时的起始地址，可用于内核加载时计算偏移量
    static payload_compiled_start: u8;
    // 每个页的开始都对齐到4K，结束并无对齐要求，结束位置应当向上取整到4K
    static srodata_page: u8;
    static erodata_page: u8;
    static sdata_page: u8;
    static edata_page: u8;
    static stext_page: u8;
    static etext_page: u8;
    // 都是u32类型，将会由r0::zero_bss每次写入一个32位零内存来初始化
    // 对应链接器脚本中的“ebss = ALIGN(4)”等等
    static mut sbss: u32;
    static mut ebss: u32;
}

/// 初始化共享调度器环境，只能由内核运行，只能运行一次
unsafe extern "C" fn init_payload_environment() -> PageList {
    // 初始化零初始段，每次写入一个u32类型的零内存
    r0::zero_bss(&mut sbss, &mut ebss);
    // 初始化堆
    let heap_start = HEAP_MEMORY.as_ptr() as usize;
    HEAP.lock().init(heap_start, HEAP_SIZE);
    // 返回一个表，表示本共享载荷应当保护的地址范围
    PageList {
        rodata: [&srodata_page, &erodata_page], // 只读
        data: [&sdata_page, &edata_page],       // 读+写
        text: [&stext_page, &etext_page],       // 只运行
    }
}

/// 共享调度器各个段的范围，方便内存管理的权限设置
///
/// 有虚拟内存，用特殊的链接器脚本，以确保对齐到4K，如果没有虚拟内存，可以使用更低的对齐方法
#[repr(C)]
pub struct PageList {
    // 这里的&'static u8指向的值并不重要，它表示的地址比较重要
    rodata: [&'static u8; 2], // 只读数据段
    data: [&'static u8; 2],   // 数据段
    text: [&'static u8; 2],   // 代码段
}
