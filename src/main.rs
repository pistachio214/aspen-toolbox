mod ssh_module;
mod aspen_module;

use crate::aspen_module::cli::{run, init_aspen};

fn main() {
    // 初始化命令行工具
    init_aspen();

    run();
}


















