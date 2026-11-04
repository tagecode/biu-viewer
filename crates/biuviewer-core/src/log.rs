use tracing_subscriber::EnvFilter;

/// 把日志写到 stderr。级别由环境变量 `RUST_LOG` 控制，未设置时默认为 `info`。
/// 重复调用是安全的，后续调用会被忽略。
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_logging_can_be_called_twice() {
        init_logging();
        init_logging();
    }
}
