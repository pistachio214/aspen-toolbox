mod ssh_module;
mod aspen_module;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
use colored::Colorize;

use crate::aspen_module::cli::{run, init_aspen};

fn main() {
    // 排除Windows、MacOs、linux之外的系统
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        eprintln!("\n[Aspen Error] => {}\n", "暂时不支 Windows、MacOs、Linux之外的系统".red());
        std::process::exit(0);
    }

    // 初始化命令行工具
    init_aspen();

    run();
}