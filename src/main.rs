use std::{
    fs::File,
    io::{Read, stdin, stdout, Write},
    net::TcpStream,
    path::Path,
    {process, thread},
};
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use prettytable::{format, row, Table};
use ssh2::{PtyModes, Session};

fn main() {
    // 构建命令详情
    let app = build_cli();
    // 获取命令集合
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("ssh", sub_matches)) => impl_ssh_action(sub_matches),
        Some(("servers", sub_matches)) => impl_servers_table_action(sub_matches),
        _ => error_action(),
    }
}

fn build_cli() -> Command {
    Command::new("aspen")
        .name("Aspen Toolbox")
        .version("0.0.1")
        .author("Aspen Soung<songyang410@outlook.com>")
        .about("Aspen工具箱")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // ssh工具箱
        .subcommand(build_ssh_toolbox())
        // 查看已配置的服务器列表
        .subcommand(build_ssh_servers_table_toolbox())
}

// 构建ssh工具的命令详情
fn build_ssh_toolbox() -> Command {
    Command::new("ssh")
        .about("ssh工具箱")
        .args([
            Arg::new("index").help("输入服务器的 序号").required(false),
        ])
}

fn build_ssh_servers_table_toolbox() -> Command {
    Command::new("servers")
        .about("查看已配置的服务器列表")
}

// 实现SSH工具
fn impl_ssh_action(matches: &ArgMatches) {
    // 清屏
    clear_terminal();

    if let Some(index) = matches.get_one::<String>("index") {
        ssh_index_action(index.clone());
    } else {
        ssh_none_index_action();
    }
}

// 实现查看已配置服务器列表
fn impl_servers_table_action(_: &ArgMatches) {
    // 清屏
    clear_terminal();

    let config_lines = get_config();
    print_services_table(config_lines);
}

fn ssh_index_action(index: String) {
    let key: i32 = match index.trim().parse() {
        Ok(num) => {
            num
        }
        Err(_) => {
            eprintln!("\n[Aspen Error] => {} 请重新输入服务器 {}:", "序号只能输入合法的整数!".red(), "序号".green());
            process::exit(1);
        }
    };

    let config_lines = get_config();
    match config_lines.get((key - 1) as usize) {
        Some(config) => {
            if !config.is_empty() {
                ssh_login(config.clone());
            }
        }
        None => {
            eprintln!("\n[Aspen Error] => {} \n", "您输入的序号超过了配置项的数量!".red());
            process::exit(1);
        }
    }
}

/**
 * 获取配置文件中的服务器数据
 */
fn get_config() -> Vec<Vec<String>> {
    let file = "./config.ini";

    // 读取服务器配置数据列表
    if Path::new(&file).exists() {
        match File::open(file) {
            Ok(mut f) => {
                let mut data = String::new();
                f.read_to_string(&mut data).expect("\n[Aspen Error] => 无法读取文件.");

                let lines: Vec<&str> = data.lines().collect();
                let mut new_lines: Vec<Vec<String>> = Vec::new();

                for line in lines {
                    let temp = line.trim();
                    // 排除空白格行
                    if temp != "" {
                        // 获取字符串的首位字符
                        let first_char = temp.chars().next().unwrap().to_string();
                        // 排除注释掉的配置信息
                        if first_char != "#" {
                            let vec = line.split(',').map(String::from).collect::<Vec<_>>();
                            new_lines.push(vec);
                        }
                    }
                }

                new_lines.clone()
            }
            Err(err) => {
                eprintln!("[Aspen Error] => Unable to read the file. {:?}", err);
                process::exit(1);
            }
        }
    } else {
        eprintln!("\n [Aspen Error] => {}\n", "您的配置文件不存在 💔".red());
        process::exit(1);
    }
}

/**
没有输入index 的情况就要打印出服务器列表,然后执行后面的操作
 */
fn ssh_none_index_action() {
    let config_lines = get_config();

    print_services_table(config_lines.clone());

    let mut config_selected: Vec<Vec<String>> = Vec::new();

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
            let config = config_lines.clone().get(index as usize).unwrap().clone();

            config_selected.push(config);
            break;
        }

        if !config_selected.is_empty() {
            ssh_login(config_selected.get(0).unwrap().clone());
        }
    }
}

fn ssh_login(config: Vec<String>) {
    let connect = format!("{}:{}", config[0].clone(), config[2].clone());
    let username = config[3].clone();
    let password = config[1].clone();
    let title = config[4].clone();

    println!("\n[Aspen Waiting] ==> 正在登录【 {} 】，请稍等...", title.green());

    let mut sess = match Session::new() {
        Ok(session) => session,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机进行 Session 链接失败！".red());
            process::exit(1);
        }
    };

    match TcpStream::connect(connect) {
        Ok(tcp) => {
            sess.set_tcp_stream(tcp);
        }
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "链接超时,请检查您的网络是否通畅或者您的Host信息是否正确".red());
            process::exit(1);
        }
    }

    match sess.handshake() {
        Ok(_) => (),
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机进行传输层协议协商失败!".red());
            process::exit(1);
        }
    }

    match sess.userauth_password(&username, &password) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("\n[Aspen Error] => {}\n", e.message().red());
            process::exit(1);
        }
    }

    let mut pty_modes = PtyModes::new();
    pty_modes.set_boolean(ssh2::PtyModeOpcode::ECHO, false); //关闭回显
    pty_modes.set_boolean(ssh2::PtyModeOpcode::IGNCR, true); //忽略输入的回车

    let mut channel = match sess.channel_session() {
        Ok(channel) => channel,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机会话通道建立失败".red());
            process::exit(1);
        }
    };

    match channel.request_pty("xterm", Some(pty_modes), None) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "与主机会话通道请求PTY失败！".red());
            process::exit(1);
        }
    }

    match channel.shell() {
        Ok(_) => {}
        Err(_) => {
            eprintln!("\n[Aspen Error] => {}\n", "启动SSH失败！".red());
            process::exit(1);
        }
    }

    match channel.handle_extended_data(ssh2::ExtendedData::Merge) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("\n[Aspen Error] => {}\n", e.message().red());
            process::exit(1);
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
        print!("\n {} \n", "Login Successful!!!".green());
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

// 打印服务器列表
fn print_services_table(lines: Vec<Vec<String>>) {
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
            let mut temp_line = line.clone();
            temp_line.insert(0, (index + 1).to_string());

            table.add_row(row![temp_line[0], temp_line[1], temp_line[3],temp_line[4],temp_line[5]]);
        }
    }

    // 打印表格到标准输出
    table.printstd();
}

//清屏
fn clear_terminal() {
    print!("\x1b[2J");
    print!("\x1b[H");
}

fn error_action() {
    eprintln!("\n[Aspen Error] => {} ", "非法指令".red(), );
    process::exit(1);
}