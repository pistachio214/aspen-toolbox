use std::io::{Read, stdin, stdout, Write};
use std::net::TcpStream;
use std::{process, thread};
use clap::ArgMatches;
use colored::Colorize;
use prettytable::{format, row, Table};
use ssh2::{PtyModes, Session};

use crate::aspen_module::config::{get_aspen_config, write_aspen_config};
use crate::ssh_module::config::{get_config, ServerConfig};

// ssh 命令实现
pub fn impl_ssh_action(matches: &ArgMatches) {
    if let Some(index) = matches.get_one::<String>("index") {
        ssh_index_action(index.clone());
    } else {
        ssh_none_index_action();
    }
}

/**
 * 实现设置服务器配置文件地址
 */
pub fn import_set_servers_path_action(matches: &ArgMatches) {
    if let Some(path) = matches.get_one::<String>("path") {
        let mut aspen_config = get_aspen_config();
        aspen_config.service_config_path = path.clone();

        match write_aspen_config(&aspen_config) {
            Ok(_) => {
                println!("\n[Aspen Success] ==> {}\n", "设置成功".green());
                process::exit(0);
            }
            Err(_) => {
                println!("\n[Aspen Error] ==> {}\n", "设置失败".red());
                process::exit(0);
            }
        }
    }
}

/**
 * 实现 ssh 命令,没有输入index
 */
fn ssh_none_index_action() {
    let config_lines = get_config();

    print_services_table(&config_lines);

    if !&config_lines.is_empty() {
        println!("请输入 {} 选择要登录的服务器:", "序号".green());
        loop {
            let mut guess = String::new();
            stdin().read_line(&mut guess).expect("读取输入错误");

            let guess: i32 = match guess.trim().parse() {
                Ok(num) => {
                    if num > config_lines.len() as i32 {
                        eprintln!("\n[Aspen Error] => {} 请重新输入服务器 {}:", "您输入的序号超过了配置项的数量!".red(), "序号".green());
                        continue;
                    }

                    num
                }
                Err(_) => {
                    eprintln!("\n[Aspen Error] => {} 请重新输入服务器 {}:", "序号只能输入合法的整数!".red(), "序号".green());
                    continue;
                }
            };

            let index = guess - 1;
            let config = config_lines.get(index as usize).unwrap();

            ssh_login(config);
            break;
        }
    }
}

/**
 * 实现 ssh 命令输入了 index 的情况
 */
fn ssh_index_action(index: String) {
    let key: i32 = match index.trim().parse() {
        Ok(num) => {
            num
        }
        Err(_) => {
            eprintln!("\n[Aspen Error] => {} 请重新输入服务器 {}:", "序号只能输入合法的整数!".red(), "序号".green());
            process::exit(0);
        }
    };

    let config_lines = get_config();
    match config_lines.get((key - 1) as usize) {
        Some(config) => {
            ssh_login(config);
        }
        None => {
            eprintln!("\n[Aspen Error] => {} \n", "您输入的序号超过了配置项的数量!".red());
            process::exit(0);
        }
    }
}

/**
 * 查看已配置服务器列表
 */
pub fn impl_servers_table_action(_: &ArgMatches) {
    let config_lines = get_config();

    print_services_table(&config_lines);
}

// 打印服务器列表
fn print_services_table(lines: &Vec<ServerConfig>) {
    // 创建表格
    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(&[
            format::LinePosition::Top,
            format::LinePosition::Title,
            format::LinePosition::Intern,
            format::LinePosition::Bottom
        ], format::LineSeparator::new('-', '+', '+', '+'))
        .padding(2, 2)
        .build();

    table.set_format(format);
    // 设置标题
    table.set_titles(row!["ID", "Host", "Port","Username","Title"]);

    // 添加行
    if !lines.is_empty() {
        for (index, line) in lines.iter().enumerate() {
            table.add_row(row![(index+1), line.host, line.port,line.username,line.title]);
        }
    }

    // 打印表格到标准输出
    table.printstd();
}

/**
 * 重点中的重点!! 实现ssh链接服务器的全部功能
 */
fn ssh_login(config: &ServerConfig) {
    let connect = format!("{}:{}", config.host.clone(), config.port.clone());
    let username = config.username.clone();
    let password = config.password.clone();
    let title = config.title.clone();

    println!("\n[Aspen Waiting] ==> 正在登录【 {} 】，请稍等...", title.green());

    let mut sess = match Session::new() {
        Ok(session) => session,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机进行 Session 链接失败！".red());
            process::exit(0);
        }
    };

    match TcpStream::connect(connect) {
        Ok(tcp) => {
            sess.set_tcp_stream(tcp);
        }
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "链接超时,请检查您的网络是否通畅或者您的Host信息是否正确".red());
            process::exit(0);
        }
    }

    match sess.handshake() {
        Ok(_) => (),
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机进行传输层协议协商失败!".red());
            process::exit(0);
        }
    }

    match sess.userauth_password(&username, &password) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("\n[Aspen Error] => {}\n", e.message().red());
            process::exit(0);
        }
    }

    let mut pty_modes = PtyModes::new();
    pty_modes.set_boolean(ssh2::PtyModeOpcode::ECHO, false); //关闭回显
    pty_modes.set_boolean(ssh2::PtyModeOpcode::IGNCR, true); //忽略输入的回车

    let mut channel = match sess.channel_session() {
        Ok(channel) => channel,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机会话通道建立失败".red());
            process::exit(0);
        }
    };

    match channel.request_pty("xterm", Some(pty_modes), None) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机会话通道请求PTY失败！".red());
            process::exit(0);
        }
    }

    match channel.shell() {
        Ok(_) => {}
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "启动SSH失败！".red());
            process::exit(0);
        }
    }

    match channel.handle_extended_data(ssh2::ExtendedData::Merge) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("\n[Aspen Error] => {}\n", e.message().red());
            process::exit(0);
        }
    }

    // 阻塞模式最后设置,避免实例化操作链接会阻塞
    sess.set_blocking(false);

    let mut ssh_stdin = channel.stream(0);

    let stdin_thread = thread::spawn(move || {
        let mut buf = [0; 1024];
        loop {
            let size = stdin().read(&mut buf).unwrap();
            ssh_stdin.write_all(&buf[..size]).unwrap();
        }
    });

    let stdout_thread = thread::spawn(move || {
        println!("\n {} \n", "Login Successful!!!".green());
        loop {
            let mut buf = [0; 1024];
            match channel.read(&mut buf) {
                Ok(c) if c > 0 => {
                    print!("{}", std::str::from_utf8(&buf).unwrap());
                    stdout().flush().unwrap();
                }
                Ok(0) => break,
                _ => (),
            }
        }

        let exit_status = match channel.exit_status() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("\n[Aspen Error] => {}\n", "主机会话状态丢失！".red());
                channel.close().unwrap();
                process::exit(0);
            }
        };

        if exit_status == 0 {
            println!("\n[Aspen Success] ==> 您已退出【 {} 】\n", title.green());
            channel.close().unwrap();
            process::exit(0);
        }
    });

    stdin_thread.join().unwrap();
    stdout_thread.join().unwrap();
}