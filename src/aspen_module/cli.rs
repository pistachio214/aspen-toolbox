use std::process;
use clap::{Arg, Command};
use colored::Colorize;

use crate::ssh_module::command::{impl_servers_table_action, impl_ssh_action, import_set_servers_path_action};

// 启动aspen命令
pub fn run() {
    // 构建命令详情
    let app = build_cli();

    // 获取命令集合
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("ssh", sub_matches)) => impl_ssh_action(sub_matches),
        Some(("servers", sub_matches)) => impl_servers_table_action(sub_matches),
        Some(("server-path", sub_matches)) => import_set_servers_path_action(sub_matches),
        _ => error_action(),
    }
}

/**
 * 构建命令
 */
pub fn build_cli() -> Command {
    // 以Cargo.toml的版本为命令的版本号
    let version = env!("CARGO_PKG_VERSION");

    Command::new("aspen")
        .name("Aspen Toolbox")
        .version(version)
        .author("Aspen Soung<songyang410@outlook.com>")
        .about("Aspen工具箱")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // ssh工具箱
        .subcommand(build_ssh_toolbox())
        // 查看已配置的服务器列表
        .subcommand(build_ssh_servers_table_toolbox())
        // 设置服务器配置文件地址
        .subcommand(build_set_servers_path_toolbox())
}

// 构建ssh工具的命令
fn build_ssh_toolbox() -> Command {
    Command::new("ssh")
        .about("ssh工具箱")
        .args([
            Arg::new("index").help("输入服务器的 序号").required(false),
        ])
}

// 构建查看服务器列表命令
fn build_ssh_servers_table_toolbox() -> Command {
    Command::new("servers")
        .about("查看已配置的服务器列表")
}

// 构建设置服务器配置地址命令
fn build_set_servers_path_toolbox() -> Command {
    let about = format!("设置服务器的配置文件地址({})", "建议绝对地址".green());

    Command::new("server-path")
        .about(about)
        .arg(Arg::new("path").help("请输入服务器配置文件地址(建议绝对地址)").required(true))
}

fn error_action() {
    eprintln!("\n[Aspen Error] => {} \n", "非法指令".red(), );
    process::exit(0);
}