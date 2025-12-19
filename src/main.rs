mod app;
mod components;  // 添加 components 模块声明

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use anyhow::Result;
use app::App;
use log::info;
use wasm_bindgen::JsValue;


fn main() -> Result<()> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
    Ok(())
}
