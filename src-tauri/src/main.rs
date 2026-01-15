// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
pub mod db;
use chrono::Local;
use log::{error, info};

fn main() {
    match svld_lib::run(){
        Ok(_) => {
            info!("时间：{}，服务启动", Local::now());
        }
        Err(e) => {
            error!("时间：{}，服务启动失败，请联系开发者解决，报错信息：{}",Local::now(), e);
        }
    }
}