mod kernel_task;
mod process;
mod executor;
mod lock;
mod shared;

pub use lock::Lock;
pub use kernel_task::{KernelTask, TaskId};
pub use process::{Process, ProcessId};
pub use executor::run_until_idle;
pub use shared::{
    SharedPayload,
    TaskState,
    kernel_should_switch
};

/// 共享调度器返回的结果
#[derive(Debug)]
#[repr(C)]
#[allow(dead_code)] // value is constructed elsewhere
pub enum TaskResult {
    /// 应当立即执行特定任务
    Task(usize),
    /// 其它地址空间的任务要运行，应当让出时间片
    ShouldYield(usize),
    /// 队列已空，所有任务已经结束
    Finished,
}
