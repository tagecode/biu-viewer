//! 协作式取消令牌。解码是同步的，无法强行中断，调用方在阶段边界检查它，
//! 让已经过期的任务尽快返回，而不是继续占用 CPU 和内存。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 可克隆的取消令牌：持有克隆的一方都能取消，所有克隆立即看到状态。
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// 标记为已取消。重复调用无副作用。
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_the_same_flag() {
        let token = CancellationToken::new();
        let clone = token.clone();
        assert!(!clone.is_cancelled());
        token.cancel();
        assert!(clone.is_cancelled());
    }

    #[test]
    fn cancel_is_idempotent() {
        let token = CancellationToken::new();
        token.cancel();
        token.cancel();
        assert!(token.is_cancelled());
    }
}
