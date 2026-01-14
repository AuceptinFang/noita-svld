// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod db;
use chrono::Local;
use log::{error, info, LevelFilter};
use env_logger::{Builder,Target};
use std::fs::OpenOptions;
use std::env;
use std::fs::File;
fn main() {
    let log_file = OpenOptions::new()
        .create(true)  // 如果文件不存在则创建
        .append(true)  // 如果文件已存在，则追加日志
        .open("app.log") // 日志文件路径
        .unwrap();
    
    setup_logger();

    match svld_lib::run(){
        Ok(_) => {
            info!("时间：{}，服务启动", Local::now());
        }
        Err(e) => {
            error!("时间：{}，服务启动失败，请联系开发者解决，报错信息：{}",Local::now(), e);
        }
    }
}

pub fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    let log_target = env::var("LOG_TARGET").unwrap_or_else(|_| "console".to_string());
    let log_file = env::var("LOG_FILE").unwrap_or_else(|_| "app.log".to_string());

    match log_target.as_str() {
        "console" => {
            env_logger::init();
        }
        "file" => {
            let file = File::create(log_file)?;
            env_logger::Builder::from_default_env()
                .target(env_logger::Target::Pipe(Box::new(file)))
                .filter(None, LevelFilter::Debug)
                .init();
        }
        _ => {
            env_logger::init(); // 默认终端
        }
    }

    Ok(())
}