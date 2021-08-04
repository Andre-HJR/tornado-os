// syscall-exit

mod config;
mod user_syscall;

pub use user_syscall::user_trap_handler;

use crate::memory::{AddressSpaceId, Satp, VirtualAddress, VirtualPageNumber};
use crate::hart::KernelHartInfo;
use bit_field::BitField;
use config::*;

pub enum SyscallResult {
    Procceed { code: usize, extra: usize },
    Retry,
    NextASID { asid: usize, satp: Satp },
    KernelTask,
    Terminate(i32),
}

impl SyscallResult {
    fn ok(extra: usize) -> Self {
        SyscallResult::Procceed { code: 0, extra }
    }
}

pub fn syscall(param: [usize; 6], user_satp: usize, func: usize, module: usize) -> SyscallResult {
    match module {
        MODULE_PROCESS => do_process(param, user_satp, func),
        MODULE_TEST_INTERFACE => do_test_interface(param, user_satp, func),
        MODULE_SWITCH_TASK => do_task(param, func),
        _ => panic!("Unknown module {:x}", module),
    }
}

fn do_task(param: [usize; 6], func: usize) -> SyscallResult {
    match func {
        FUNC_SWITCH_TASK => switch_next_task(param[0]),
        _ => unimplemented!()
    }
}


/// 用户态轮询任务的时候，发现下一个任务在不同地址空间，则产生该系统调用
/// 从共享调度器里面拿出下一个任务的引用，根据地址空间编号切换到相应的地址空间
/// 下一个任务的地址空间编号由用户通过 a0 参数传给内核
fn switch_next_task(next_asid: usize) -> SyscallResult {
    println!("next asid: {}", next_asid);
    if next_asid == 0 {
        // 如果是内核任务
        SyscallResult::KernelTask
    } else {
        let satp = KernelHartInfo::user_satp(next_asid).expect("get satp register with asid");
        SyscallResult::NextASID { asid: next_asid, satp }
    }
}

fn do_process(param: [usize; 6], user_satp: usize, func: usize) -> SyscallResult {
    match func {
        FUNC_PROCESS_EXIT => SyscallResult::Terminate(param[0] as i32),
        FUNC_PROCESS_PANIC => {
            //[line as usize, col as usize, f_buf, f_len, m_buf, m_len]
            let [line, col, f_buf, f_len, m_buf, m_len] = param;
            let user_satp = crate::memory::Satp::new(user_satp);
            let file_name = if f_buf == 0 {
                None
            } else {
                let slice = unsafe { get_user_buf(user_satp, f_buf, f_len) };
                Some(core::str::from_utf8(slice).unwrap())
            };
            let msg = if m_buf == 0 {
                None
            } else {
                let slice = unsafe { get_user_buf(user_satp, m_buf, m_len) };
                Some(core::str::from_utf8(slice).unwrap())
            };
            let file_name = file_name.unwrap_or("<no file>");
            let msg = msg.unwrap_or("<no message>");
            println!(
                "[Kernel] User process panicked at '{}', {}:{}:{}",
                msg, file_name, line, col
            );
            SyscallResult::Terminate(-1)
        }
        _ => panic!(
            "Unknown syscall process, func: {}, param: {:?}",
            func, param
        ),
    }
}

fn do_test_interface(param: [usize; 6], user_satp: usize, func: usize) -> SyscallResult {
    match func {
        FUNC_TEST_WRITE => {
            let (_iface, buf_ptr, buf_len) = (param[0], param[1], param[2]); // 调试接口编号，缓冲区指针，缓冲区长度
            let user_satp = crate::memory::Satp::new(user_satp);
            let slice = unsafe { get_user_buf(user_satp, buf_ptr, buf_len) };
            for &byte in slice {
                crate::sbi::console_putchar(byte as usize);
            }
            SyscallResult::Procceed {
                code: 0,
                extra: buf_len,
            }
        }
        FUNC_TEST_READ_LINE => {
            // 读入len个字符，如果遇到换行符，或者缓冲区满，就停止
            let (_iface, buf_ptr, buf_len) = (param[0], param[1], param[2]); // 调试接口编号，输出缓冲区指针，输出缓冲区长度
            let user_satp = crate::memory::Satp::new(user_satp);
            let slice = unsafe { get_user_buf_mut(user_satp, buf_ptr, buf_len) };
            for i in 0..buf_len {
                let input = crate::sbi::console_getchar();
                let byte = input as u8; // 假定SBI输入都是u8类型
                if byte == b'\n' {
                    break;
                }
                slice[i] = byte;
            }
            SyscallResult::Procceed {
                code: 0,
                extra: buf_len,
            }
        }
        _ => panic!("Unknown syscall test, func: {}, param: {:?}", func, param),
    }
}

unsafe fn get_user_buf<'a>(user_satp: Satp, buf_ptr: usize, buf_len: usize) -> &'a [u8] {
    get_user_buf_mut(user_satp, buf_ptr, buf_len)
}

unsafe fn get_user_buf_mut<'a>(user_satp: Satp, buf_ptr: usize, buf_len: usize) -> &'a mut [u8] {
    let offset = buf_ptr.get_bits(0..12); // Sv39 里面虚拟地址偏移量为低 12 位
    let vpn = VirtualPageNumber::floor(VirtualAddress(buf_ptr));
    let ppn = user_satp.translate(vpn).expect("no page fault");
    let va = ppn
        .start_address()
        .virtual_address_linear()
        .0
        .wrapping_add(offset);
    let ptr = (va as *const u8).as_ref().expect("non-null pointer");
    core::slice::from_raw_parts_mut(ptr as *const _ as *mut _, buf_len)
}
