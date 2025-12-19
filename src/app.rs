use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use crate::components::Path;
use crate::components::Backups;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    let is_valid = use_state(|| false);

    let on_valid_change = {
        let is_valid = is_valid.clone();
        Callback::from(move |valid: bool| {
            is_valid.set(valid);
        })
    };

    html! {
        <main class="container">
            <h1>{"Noita 存档管理器"}</h1>
            // 路径选择组件
            <Path on_valid_change={on_valid_change} />

            <Backups is_valid={*is_valid} />
        </main>
    }
}

