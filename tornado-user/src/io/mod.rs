//! 协程内核环境下的用户程序文件 IO 库

use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;
use crate::syscall::sys_enroll_write;
use super::syscall::sys_enroll_read;
pub struct PollTwice {
    first: bool
}

impl PollTwice {
    pub fn new() -> Self {
        Self { first: true }
    }
}

impl Future for PollTwice {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.first {
            println!("poll first");
            self.first = false;
            Poll::Pending
        } else {
            println!("poll second");
            Poll::Ready(())
        }
    }
}

pub fn read_block(block_id: usize, buf: &mut [u8]) -> PollTwice {
    let sys_ret = sys_enroll_read(block_id, buf);
    PollTwice::new()
}

pub fn write_block(block_id: usize, buf: &[u8]) -> PollTwice {
    let sys_ret = sys_enroll_write(block_id, buf);
    PollTwice::new()
}