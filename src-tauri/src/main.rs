// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod db;
use chrono::Local;
use log::{error, info, LevelFilter};
use env_logger::{Builder,Target};
use std::fs::OpenOptions;

fn main() {
    let log_file = OpenOptions::new()
        .create(true)  // 如果文件不存在则创建
        .append(true)  // 如果文件已存在，则追加日志
        .open("app.log") // 日志文件路径
        .unwrap();

    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.target(Target::Pipe(Box::new(log_file)));

    builder.filter(None, LevelFilter::Debug);
    builder.init();

    match svld_lib::run(){
        Ok(_) => {
            info!("时间：{}，服务启动", Local::now());
        }
        Err(e) => {
            error!("时间：{}，服务启动失败，请联系开发者解决，报错信息：{}",Local::now(), e);
        }
    }
}
