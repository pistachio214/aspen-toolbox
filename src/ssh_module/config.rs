use std::{fs::File, io::Read, path::Path, process};
use std::path::PathBuf;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::aspen_module::config::{get_aspen_config};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub title: String,
    pub category: String,
    pub host: String,
    pub port: u32,
    pub username: String,
    pub password: String,
    pub remark: String,
}

/**
 * 获取配置文件中的服务器数据
 */
pub fn get_config() -> Vec<ServerConfig> {
    let aspen_config = get_aspen_config();
    let file_path = aspen_config.service_config_path;

    if file_path.is_empty() {
        eprintln!("\n [Aspen Error] => {}\n", "💔 您的配置文件地址尚未设置,请先执行命令 set-path 设置! ".red());
        process::exit(0);
    }

    let file = PathBuf::from(file_path.clone());

    // 读取服务器配置数据列表
    if Path::new(&file).exists() {
        let configs = read_server_config(&file).unwrap();
        configs
    } else {
        eprintln!("\n [Aspen Error] => {}\n", format!("💔 您的配置文件({})不存在 ", file_path).red());
        process::exit(0);
    }
}

fn read_server_config(file_path: &PathBuf) -> Result<Vec<ServerConfig>, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open(&file_path) {
        // 读取文件内容
        let mut contents = String::new();

        match file.read_to_string(&mut contents) {
            Ok(_) => {
                // 使用 serde_json 解析 JSON
                let config: Vec<ServerConfig> = serde_json::from_str(&contents).unwrap();

                Ok(config)
            }
            Err(_) => {
                eprintln!("\n[Aspen Error] => {} \n", "读取配置json文件失败!".red());
                process::exit(0);
            }
        }
    } else {
        eprintln!("\n[Aspen Error] => {} \n", "无法打开配置文件!".red());
        process::exit(0);
    }
}
