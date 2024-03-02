use std::{fs, process};
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions};
use std::io::prelude::*;
use colored::Colorize;
use clap::{Arg, Command};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use crate::ssh_module::command::{
    impl_servers_table_action, impl_ssh_action,
    import_get_servers_path_action,
    import_set_servers_path_action,
};

pub fn get_home_dir() -> PathBuf {
    let home_dir = match dirs::home_dir() {
        None => {
            eprintln!("\n[Aspen Error] => {} \n", "系统主目录获取失败".red(), );
            process::exit(0);
        }
        Some(dir) => dir
    };

    home_dir
}

pub fn init_aspen() {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let dir = env!("CARGO_PKG_NAME");

        let shell_dir = format!("{}/{}/shell", get_home_dir().to_str().unwrap().to_string(), dir.to_string());
        generate_folder(shell_dir.clone());

        let controller_path = shell_dir.clone() + "/controller.sh";
        // 要写入的内容
        let controller_content = "#!/bin/bash\n\ncurrent_dir=$(dirname \"$(realpath \"$0\")\")\n\neval \"$(which expect) $current_dir/script.ex $1 $2 $3 $4 $5\"";

        generate_shell(controller_path, controller_content);

        let script_path = shell_dir.clone() + "/script.ex";
        let script_content = "#!/usr/bin/expect\n\nset SERVER_NAME [lindex $argv 0]\nset IP [lindex $argv 1]\nset PORT [lindex $argv 2]\nset USER_NAME [lindex $argv 3]\nset PASSWORD [lindex $argv 4]\n\nspawn ssh -p $PORT $USER_NAME@$IP\n\nexpect {\n    -timeout 300\n    \"*assword\" { send \"$PASSWORD\\r\\n\"; exp_continue ; sleep 3; }\n    \"yes/no\" { send yes\\n\"; exp_continue; }\n    \"Last*\" {\n        puts \"\\nLogin Successful!!!\\n\";\n    }\n    timeout { puts \"Expect was timeout.\"; return }\n}\n\ninteract\n";

        generate_shell(script_path, script_content);
    }
}

// 启动aspen命令
pub fn run() {
    // 构建命令详情
    let app = build_cli();

    // 获取命令集合
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("ssh", sub_matches)) => impl_ssh_action(sub_matches),
        Some(("all", sub_matches)) => impl_servers_table_action(sub_matches),
        Some(("set-path", sub_matches)) => import_set_servers_path_action(sub_matches),
        Some(("get-path", sub_matches)) => import_get_servers_path_action(sub_matches),
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
        // 获取服务器配置文件地址
        .subcommand(build_get_servers_path_toolbox())
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
    Command::new("all")
        .about("查看已配置的服务器列表")
}

// 构建设置服务器配置地址命令
fn build_set_servers_path_toolbox() -> Command {
    let about = format!("设置服务器的配置文件地址({})", "建议绝对地址".green());

    Command::new("set-path")
        .about(about)
        .arg(Arg::new("path").help("请输入服务器配置文件地址(建议绝对地址)").required(true))
}

// 构建获取服务器配置地址命令
fn build_get_servers_path_toolbox() -> Command {
    Command::new("get-path")
        .about("获取服务器的配置文件地址")
}

fn error_action() {
    eprintln!("\n[Aspen Error] => {} \n", "非法指令".red(), );
    process::exit(0);
}

// 构建存储文件夹
pub fn generate_folder(folder_path: String) {
    // 检查文件夹是否存在，如果不存在则创建
    if !Path::new(&folder_path).exists() {
        fs::create_dir_all(&folder_path).unwrap();
        match fs::create_dir_all(&folder_path) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("\n[Aspen Error] => 创建文件夹 {} 失败！ \n", folder_path.red(), );
                process::exit(0);
            }
        }

        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            // 获取目标文件夹信息
            let metadata = match fs::metadata(&folder_path) {
                Ok(meta) => meta,
                Err(_) => {
                    eprintln!("\n[Aspen Error] => 获取目标文件夹 {} 相关信息失败！ \n", folder_path.red(), );
                    process::exit(0);
                }
            };

            // 设置文件夹权限为 775
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o775);
            match fs::set_permissions(&folder_path, permissions) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("\n[Aspen Error] => 设置文件夹 {} 权限失败！ \n", folder_path.red(), );
                    process::exit(0);
                }
            }
        }
    }
}

// 构建脚本
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn generate_shell(file_path: String, content: &str) {
    match fs::metadata(&file_path) {
        Ok(_) => {}
        Err(_) => {
            // 创建文件并打开以进行写入，如果文件不存在则会创建它
            let mut file = match OpenOptions::new()
                .write(true)
                .create(true)
                .mode(0o775) // 设置权限为 775
                .open(&file_path) {
                Ok(file) => file,
                Err(_) => {
                    eprintln!("\n[Aspen Error] => {} \n", "创建脚本失败".red(), );
                    process::exit(0);
                }
            };

            // 将内容写入文件
            match file.write_all(content.as_bytes()) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("\n[Aspen Error] => {} \n", "写入脚本内容失败！".red());
                    process::exit(0);
                }
            }
        }
    }
}