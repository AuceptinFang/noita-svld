use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct Empty {}

#[function_component(Version)]
pub fn version() -> Html {
    let checking = use_state(|| false);
    let message = use_state(|| String::from(""));
    let current_version = use_state(|| String::from("加载中..."));

    {
        let current_version = current_version.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match invoke("get_version", JsValue::NULL).await.as_string() {
                    Some(v) => current_version.set(v),
                    None => current_version.set("未知版本".to_string()),
                }
            });
            || {}
        });
    }


    let on_check_update = {
        let checking = checking.clone();
        let message = message.clone();
        
        Callback::from(move |_| {
            let checking = checking.clone();
            let message = message.clone();
            
            checking.set(true);
            message.set(String::from("正在检查更新..."));
            
            spawn_local(async move {
                let args = serde_wasm_bindgen::to_value(&Empty {}).unwrap();
                let result = invoke("check_update", args).await;
                
                match serde_wasm_bindgen::from_value::<String>(result) {
                    Ok(msg) => message.set(msg),
                    Err(_) => message.set(String::from("检查更新失败")),
                }
                
                checking.set(false);
            });
        })
    };

    html! {
        <div class="version-card">
            <div class="version-row">
                <div class="version-text">
                    {"当前版本： "}
                    <span class="version-number">{ &*current_version }</span>
                </div>

                <button
                    onclick={on_check_update}
                    disabled={*checking}
                    class="update-button"
                >
                    { if *checking { "检查中..." } else { "检查更新" } }
                </button>
            </div>

            if !message.is_empty() {
                <div class="update-message">
                    { &*message }
                </div>
            }
        </div>
    }
}
