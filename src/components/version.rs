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
        <div class="version-container">
            <button 
                onclick={on_check_update} 
                disabled={*checking}
                class="update-button"
            >
                { if *checking { "检查中..." } else { "检查更新" } }
            </button>
            
            if !message.is_empty() {
                <p class="update-message">{ &*message }</p>
            }
        </div>
    }
}
