use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    PrintVersion,
    PrintHelp,
    /// 启动窗口，可选带一个图片或目录路径。
    Launch(Option<PathBuf>),
}

pub fn version_line() -> String {
    format!("biuviewer {}", env!("CARGO_PKG_VERSION"))
}

pub fn help_text() -> String {
    format!(
        "{}\n\n用法:\n  biuviewer [选项] [路径]\n\n参数:\n  路径            要打开的图片或目录，支持相对路径\n\n选项:\n  -h, --help      显示帮助\n  -V, --version   显示版本\n",
        version_line()
    )
}

pub fn parse_args<S: AsRef<str>>(args: &[S]) -> CliAction {
    let mut path = None;
    for arg in args {
        match arg.as_ref() {
            "--version" | "-V" => return CliAction::PrintVersion,
            "--help" | "-h" => return CliAction::PrintHelp,
            other if path.is_none() => path = Some(PathBuf::from(other)),
            _ => {}
        }
    }
    CliAction::Launch(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_line_matches_package_manifest() {
        assert_eq!(
            version_line(),
            format!("biuviewer {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn version_flag_does_not_launch() {
        assert_eq!(parse_args(&["--version"]), CliAction::PrintVersion);
        assert_eq!(parse_args(&["-V"]), CliAction::PrintVersion);
    }

    #[test]
    fn help_flag_does_not_launch() {
        assert_eq!(parse_args(&["--help"]), CliAction::PrintHelp);
        assert_eq!(parse_args(&["-h"]), CliAction::PrintHelp);
    }

    #[test]
    fn no_args_launches_the_app() {
        let args: [&str; 0] = [];
        assert_eq!(parse_args(&args), CliAction::Launch(None));
    }

    #[test]
    fn first_positional_path_wins() {
        assert_eq!(
            parse_args(&["photos", "image.png"]),
            CliAction::Launch(Some(PathBuf::from("photos")))
        );
    }

    #[test]
    fn flags_win_over_paths_regardless_of_order() {
        assert_eq!(
            parse_args(&["image.png", "--version"]),
            CliAction::PrintVersion
        );
    }
}
