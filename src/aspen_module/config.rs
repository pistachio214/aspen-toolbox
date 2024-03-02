use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    io::{Read, Write},
};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::io::prelude::*;

use crate::aspen_module::cli::{generate_folder, get_home_dir};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub service_config_path: String,
}

/**
 * 获取Aspen的配置文件地址
 */
pub fn get_aspen_config() -> Config {
    let dir = env!("CARGO_PKG_NAME");
    let config_dir = format!("{}/{}/config", get_home_dir().to_str().unwrap().to_string(), dir.to_string());

    generate_folder(config_dir.clone());

    // 命令指定的配置文件位置
    let config_path = format!("{}/aspen_config.json", config_dir);

    //配置文件存在就直接读取,反之则创建
    if Path::new(&config_path).exists() {
        // 打开文件
        read_config(&PathBuf::from(config_path.as_str())).unwrap()
    } else {
        // 创建一个 Config 结构体实例
        let config = Config {
            service_config_path: "".to_string(),
        };

        // 将 Config 结构体序列化为 JSON 格式的字符串
        let json_string: String = match serde_json::to_string_pretty(&config) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("\n[Aspen Error] => {} \n", "错误配置信息!".red());
                std::process::exit(0);
            }
        };

        if let Ok(mut file) = File::create(&config_path) {
            file.write_all(json_string.as_bytes()).unwrap();
            // println!("成功创建并写入 JSON 文件: {:?}", config_path);

            config
        } else {
            eprintln!("\n[Aspen Error] => {} \n", "无法创建配置文件!".red());
            std::process::exit(0);
        }
    }
}


/**
 * 读取指定位置的json文件内容
 */
pub fn read_config(file_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    if let Ok(mut file) = File::open(&file_path) {
        // 读取文件内容
        let mut contents = String::new();

        match file.read_to_string(&mut contents) {
            Ok(_) => {
                // 使用 serde_json 解析 JSON
                Ok(serde_json::from_str(&contents).unwrap())
            }
            Err(_) => {
                eprintln!("\n[Aspen Error] => {} \n", "读取配置json文件失败!".red());
                std::process::exit(0);
            }
        }
    } else {
        eprintln!("\n[Aspen Error] => {} \n", "无法打开配置文件!".red());
        std::process::exit(0);
    }
}

/**
 * 写入指定文件内容
 */
pub fn write_aspen_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let dir = env!("CARGO_PKG_NAME");
    let config_path = format!("{}/{}/config/aspen_config.json", get_home_dir().to_str().unwrap().to_string(), dir.to_string());

    let mut file = match OpenOptions::new().write(true).truncate(true).open(config_path) {
        Ok(f) => f,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {} \n", "打开指定配置文件错误!".red());
            std::process::exit(0);
        }
    };

    let json_string = match serde_json::to_string_pretty(config) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("\n[Aspen Error] => {} \n", "写入的配置信息转为json失败!".red());
            std::process::exit(0);
        }
    };

    match file.write_all(json_string.as_bytes()) {
        Ok(_) => {
            Ok(())
        }
        Err(_) => {
            eprintln!("\n[Aspen Error] => {} \n", "写入Aspen配置文件失败!".red());
            std::process::exit(0);
        }
    }
}
