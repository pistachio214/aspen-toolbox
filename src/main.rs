use std::fs::File;
use std::path::Path;
use clap::{ArgMatches, Command};
use prettytable::{format, row, Table};
use ssh2::PtyModes;
use {
    ssh2::Session,
    std::{
        io::{stdin, stdout, Read, Write},
        net::TcpStream,
        io, process, thread,
    },
};

fn main() {
    // 构建命令详情
    let app = build_cli();
    // 获取命令集合
    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("ssh", sub_matches)) => impl_ssh_action(sub_matches),
        _ => error_action(),
    }
}

fn build_cli() -> Command {
    Command::new("aspen")
        .name("aspen toolbox")
        .version("0.0.1")
        .author("Aspen Soung<songyang420@outlook.com>")
        .about("尝试写的一个由Rust开发的命令行工具")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(build_ssh_toolbox())
}

// 构建ssh工具的命令详情
fn build_ssh_toolbox() -> Command {
    Command::new("ssh")
        .about("打印服务器列表")
    // .args([
    //     Arg::new("input").help("输入的是啥").required(true),
    //     Arg::new("name").help("输入名称").required(true)
    // ])
}

// 试试这个实现
fn impl_ssh_action(_: &ArgMatches) {
    let file = "./config.ini";

    // 1. 读取服务器配置数据列表
    if Path::new(&file).exists() {
        match File::open(file) {
            Ok(mut f) => {
                let mut data = String::new();
                f.read_to_string(&mut data).expect("[Aspen Error] => 无法读取文件.");

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

                print_services_table(new_lines.clone());

                let mut config_selected: Vec<Vec<String>> = Vec::new();

                if !&new_lines.is_empty() {
                    println!("请输入 序号 选择要登录的服务器:");
                    loop {
                        let mut guess = String::new();
                        io::stdin().read_line(&mut guess).expect("读取输入错误");

                        let guess: i32 = match guess.trim().parse() {
                            Ok(num) => {
                                if num > new_lines.len() as i32 {
                                    eprintln!("您输入的序号超过了配置项的数量!");
                                    continue;
                                }

                                num
                            }
                            Err(_) => {
                                eprintln!("序号只能输入合法的整数!");
                                continue;
                            }
                        };

                        let index = guess - 1;
                        let config = new_lines.clone().get(index as usize).unwrap().clone();

                        config_selected.push(config);
                        break;
                    }

                    if !config_selected.is_empty() {
                        ssh_login(config_selected.get(0).unwrap().clone());
                    }
                }
            }
            Err(err) => {
                eprintln!("[Aspen Error] => Unable to read the file. {:?}", err);
                process::exit(1);
            }
        }
    } else {
        eprintln!("[Aspen Error] => 您的配置文件不存在 💔");
        process::exit(1);
    }
}

fn ssh_login(config: Vec<String>) {
    println!("选中的配置是: {:?}", config);

    let connect = format!("{}:{}", config[0].clone(), config[2].clone());
    let username = config[3].clone();
    let password = config[1].clone();

    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(TcpStream::connect(connect).unwrap());
    sess.handshake().unwrap();
    sess.userauth_password(&username, &password).unwrap();

    let mut pty_modes = PtyModes::new();
    pty_modes.set_boolean(ssh2::PtyModeOpcode::ECHO,false); //关闭回显
    pty_modes.set_boolean(ssh2::PtyModeOpcode::IGNCR, true); //忽略输入的回车

    let mut channel = sess.channel_session().unwrap();
    channel.request_pty("xterm", Some(pty_modes), None).unwrap();
    channel.shell().unwrap();
    channel.handle_extended_data(ssh2::ExtendedData::Merge).unwrap();

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


        channel.close().unwrap();
        print!("Exit: {}", channel.exit_status().unwrap());
    });

    // let status = sess.wait_status().unwrap();

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

fn error_action() {
    println!("发生了错误")
}