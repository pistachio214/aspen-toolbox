use std::{
    io::{stdin, Write},
    process,
};
use clap::ArgMatches;
use colored::Colorize;
use prettytable::{format, row, Table};

#[cfg(any(target_os = "macos", target_os = "linux"))]
use crate::aspen_module::cli::get_home_dir;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::{env, io};

#[cfg(target_os = "windows")]
use std::{net::TcpStream, thread, io::{stdout, Read}, time::Duration};
#[cfg(target_os = "windows")]
use ssh2::{PtyModes, Session, FLUSH_ALL};
#[cfg(target_os = "windows")]
use crossterm::event::{read, Event, KeyCode, KeyEventKind, KeyModifiers};

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
 * 实现获取服务器配置文件地址
 */
pub fn import_get_servers_path_action(_: &ArgMatches) {
    let aspen_config = get_aspen_config();

    println!("\n[Aspen Success] ==> Servers Path: {}\n", aspen_config.service_config_path.green());
    process::exit(0);
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
                    if num < 1 {
                        eprintln!("\n[Aspen Error] => {} 请重新输入服务器 {}:", "序号必须大于 0 !".red(), "序号".green());
                        continue;
                    }

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
            if num < 1 {
                eprintln!("\n[Aspen Error] => {} 请重新输入服务器 {}:", "序号必须大于 0 !".red(), "序号".green());
                process::exit(0);
            }

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
            // 清屏
            clear_terminal();

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
    table.set_titles(row![
        "ID","Title","Host","Port","Server Type","Username","Remark"
    ]);

    // 添加行
    if !lines.is_empty() {
        for (index, line) in lines.iter().enumerate() {
            table.add_row(row![
                (index+1),line.title, line.host, line.port,
                line.category,line.username,line.remark
            ]);
        }
    }

    // 清屏
    clear_terminal();

    // 打印表格到标准输出
    table.printstd();
}

/**
 * 重点中的重点!! 实现ssh链接服务器的全部功能
 */
fn ssh_login(config: &ServerConfig) {
    // macOS 平台下编译
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        ssh_login_macos_and_linux(&config);
    }

    // windows平台下处理
    #[cfg(target_os = "windows")]
    {
        ssh_login_windows(&config)
    }
}

// windows 系统中,目前使用ssh2来处理远程登录问题
#[cfg(target_os = "windows")]
fn ssh_login_windows(config: &ServerConfig) {
    let connect = format!("{}:{}", config.host.clone(), config.port.clone());
    let username = config.username.clone();
    let password = config.password.clone();
    let title = config.title.clone();

    println!("\n[Aspen Waiting] ==> 正在登录【 {} 】，请稍等...", config.title.clone().green());

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
    pty_modes.set_u32(ssh2::PtyModeOpcode::TTY_OP_OSPEED, 115200);
    pty_modes.set_u32(ssh2::PtyModeOpcode::TTY_OP_ISPEED, 115200);

    let mut channel = match sess.channel_session() {
        Ok(channel) => channel,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机会话通道建立失败".red());
            process::exit(0);
        }
    };

    match channel.request_pty("xterm-256color", Some(pty_modes), Some((80, 24, 0, 0))) {
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
        loop {
            let mut buf = [0; 4096];
            let data = {
                match read().unwrap() {
                    Event::Key(e) if matches!(e.kind, KeyEventKind::Press) => match e.code {
                        KeyCode::Char(c) => match e.modifiers {
                            KeyModifiers::CONTROL => {
                                buf[0] = c as u8 - 96;
                                &buf[..1]
                            }
                            KeyModifiers::NONE => c.encode_utf8(&mut buf).as_bytes(),
                            _ => c.encode_utf8(&mut buf).as_bytes(),
                        },
                        _ => match e.code {
                            KeyCode::Enter => "\n",
                            KeyCode::Backspace => "\x08",
                            KeyCode::Tab => "\x09",
                            KeyCode::Esc => "\x1b",
                            KeyCode::Home => "\x1b[H",
                            KeyCode::End => "\x1b[F",
                            KeyCode::Insert => "\x1b\x5b\x32\x7e",
                            KeyCode::Delete => "\x1b\x5b\x33\x7e",
                            KeyCode::Left => "\x1b[D",
                            KeyCode::Up => "\x1b[A",
                            KeyCode::Right => "\x1b[C",
                            KeyCode::Down => "\x1b[B",
                            _ => todo!(),
                        }.as_bytes(),
                    },
                    _ => continue,
                }
            };

            ssh_stdin.write_all(&data).unwrap();
        };
    });

    let stdout_thread = thread::spawn(move || {
        println!("\n {} \n", "Login Successful!!!".green());
        loop {
            let mut buf = [0; 4096];
            match channel.read(&mut buf) {
                Ok(c) if c > 0 => {
                    print!("{}", std::str::from_utf8(&buf[..c]).unwrap());
                    channel.stream(FLUSH_ALL).flush().unwrap();
                    stdout().flush().unwrap();
                }
                Ok(0) => break,
                _ => thread::sleep(Duration::from_millis(1)),
            };
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

// macos linux 系统中,ssh登录的实现(采用脚本命令去处理,解决ssh2中命令tab和vim编码问题)
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn ssh_login_macos_and_linux(config: &ServerConfig) {
    println!("\n[Aspen Waiting] ==> 正在登录【 {} 】，请稍等...", config.title.clone().green());

    let dir = env!("CARGO_PKG_NAME");
    let controller_path = format!("{}/{}/shell/controller.sh", get_home_dir().to_str().unwrap().to_string(), dir.to_string());

    // 参数列表 (名称,IP,Port,用户名,密码)
    let args = vec![
        "-e".to_string(),
        controller_path,
        config.title.clone(),
        config.host.clone(),
        config.port.clone().to_string(),
        config.username.clone(),
        config.password.to_string(),
    ];

    // 执行用户输入的命令
    let mut child = process::Command::new("sh")
        .args(&args)
        .spawn()
        .expect("执行命令失败");

    // 从子进程的 stdin 获取一个写入器
    if let Some(mut stdin) = child.stdin.take() {
        // 向子进程发送输入
        writeln!(stdin, "echo 'Hello, World!'").unwrap();
    }

    // 从子进程的 stdout 获取一个读取器
    if let Some(mut stdout) = child.stdout.take() {
        // 从子进程读取输出
        match io::copy(&mut stdout, &mut io::stdout()) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("\n[Aspen Error] => {}\n", "从子进程读取输出失败！".red());
                process::exit(0);
            }
        }
    }

    // 从子进程的 stderr 获取一个读取器
    if let Some(mut stderr) = child.stderr.take() {
        // 从子进程读取错误信息
        match io::copy(&mut stderr, &mut io::stderr()) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("\n[Aspen Error] => {}\n", "从子进程读取错误信息失败！".red());
                process::exit(0);
            }
        }
    }

    // 等待子进程执行完毕
    child.wait().unwrap();

    println!("\n[Aspen Success] ==> 您已退出【 {} 】\n", config.title.green());
    process::exit(0);
}

//清屏
fn clear_terminal() {
    print!("\x1b[2J");
    print!("\x1b[H");
}