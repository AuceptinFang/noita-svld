use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use yew::{function_component, html, Callback, Html};
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[function_component(Log)]
pub fn log() -> Html {
    let on_open_log = Callback::from(|_| {
        spawn_local(async {
            invoke("open_log", JsValue::NULL).await;
        });
    });

    html! {
        <div class="settings-group">
            <div class="setting-card">
                <div class="setting-text">
                    <span class="label">{"运行日志"}</span>
                    <p class="description">{"反馈bug时请提供日志"}</p>
                </div>

                <button class="btn-log" onclick={on_open_log}>
                    <span class="btn-emoji">{"📂"}</span>
                    {"打开日志"}
                </button>
            </div>
        </div>
    }
}