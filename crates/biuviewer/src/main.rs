#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match biuviewer::parse_args(&args) {
        biuviewer::CliAction::PrintVersion => {
            println!("{}", biuviewer::version_line());
        }
        biuviewer::CliAction::PrintHelp => {
            println!("{}", biuviewer::help_text());
        }
        biuviewer::CliAction::Launch(path) => biuviewer::run(path),
    }
}
