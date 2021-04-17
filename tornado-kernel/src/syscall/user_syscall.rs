//! 从用户过来的系统调用在这里处理
use riscv::register::scause::{self, Trap, Interrupt};
use riscv::register::{sepc, stval};
use crate::memory::{self, PAGE_SIZE};
use crate::trap;
use super::{SyscallResult, syscall};

/// 测试用的中断处理函数，用户态发生中断会陷入到这里
/// 用户地址空间的 satp 寄存器通过 t2 传给内核
#[export_name = "_user_trap_handler"]
pub extern "C" fn user_trap_handler() {
    let user_satp: usize;
    unsafe {
        asm!("mv {}, t2", out(reg) user_satp, options(nomem, nostack));
    }
    let user_satp = memory::Satp::new(user_satp);
    let swap_cx_va = memory::VirtualAddress(memory::SWAP_CONTEXT_VA);
    let swap_cx_vpn = memory::VirtualPageNumber::floor(swap_cx_va);
    let swap_cx_ppn = user_satp
        .translate(swap_cx_vpn)
        .unwrap();
    // 将物理页号转换成裸指针
    let swap_cx = unsafe {
        (swap_cx_ppn.start_address()
            .0
            .wrapping_add(memory::KERNEL_MAP_OFFSET) as *mut trap::SwapContext)
            .as_mut()
            .unwrap()
    };
    // 从 SwapContext 中读东西
    let a7 =swap_cx.x[16];
    let a6 =swap_cx.x[15];
    let a0 = swap_cx.x[9];
    let a1 = swap_cx.x[10];
    match scause::read().cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            println!("s mode timer!");
            // 目前遇到时钟中断先让系统退出，等把内核完善好了再来处理
            crate::sbi::shutdown();
        },
        Trap::Exception(scause::Exception::Breakpoint) => {
            println!("user mode panic!");
            crate::sbi::shutdown();
        },
        Trap::Exception(scause::Exception::UserEnvCall) => {
            let func = a6;
            let param = [a0, a1];
            match syscall(param, func, a7) {
                SyscallResult::Procceed { code,  extra} => {
                    swap_cx.x[9] = code;
                    swap_cx.x[10] = extra;
                    swap_cx.epc += 4;        
                },
                SyscallResult::Retry => {
                    // 不跳过指令，继续运行
                }
            }
            trap::switch_to_user(swap_cx, user_satp.inner())
            // unreachable!()
        }
        _ => todo!("scause: {:?}, sepc: {:#x}, stval: {:#x}", scause::read().cause(), sepc::read(), stval::read())
    }
}

