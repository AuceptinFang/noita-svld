use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use web_sys::console;

#[derive(Properties, PartialEq)]
pub struct PathProps {
    pub on_valid_change: Callback<bool>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize)]
struct SavePathArgs {
    path: String,
}

#[function_component(Path)]
pub fn path() -> Html {
    // 默认显示的提示文本
    let current_path = use_state(|| "正在检测存档路径...".to_string());
    let is_valid = use_state(|| false);

    // 初始化检测逻辑
    {
        let current_path = current_path.clone();
        let is_valid = is_valid.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = invoke("get_save_path", JsValue::NULL).await;
                match response {
                    Ok(value) => {
                        if let Some(path) = value.as_string() {
                            current_path.set(path);
                            
                            // Try to verify the path
                            match invoke("verify_validation", JsValue::NULL).await {
                                Ok(_) => {
                                    console::log_1(&"验证成功".into());
                                    is_valid.set(true);
                                }
                                Err(e) => {
                                    console::log_1(&format!("验证失败：{:?}", e).into());
                                    is_valid.set(false);
                                }
                            }
                        } else {
                            current_path.set("未设置路径".to_string());
                            is_valid.set(false);
                        }
                    }
                    Err(_) => {
                        current_path.set("未设置路径".to_string());
                        is_valid.set(false);
                    }
                }
            });
            || {}
        });
    }

    // 浏览文件夹
    let on_select_folder = {
        let current_path = current_path.clone();
        let is_valid = is_valid.clone();

        Callback::from(move |_: MouseEvent| {
            let current_path = current_path.clone();
            let is_valid = is_valid.clone();
            spawn_local(async move {
                // 调用 Tauri 的选择文件夹弹窗
                let response = invoke("select_save_path", JsValue::NULL).await;
                match response {
                    Ok(value) => {
                        if let Some(path) = value.as_string() {
                            // 1. 更新 UI 显示
                            current_path.set(path.clone());

                            // 2. 保存到后端环境
                            let args = serde_wasm_bindgen::to_value(&SavePathArgs { path: path.clone() }).unwrap();
                            let _ = invoke("save_path_to_env", args).await;

                            // 3. 再次验证有效性
                            match invoke("verify_validation", JsValue::NULL).await {
                                Ok(_) => {
                                    console::log_1(&"验证成功".into());
                                    is_valid.set(true);
                                }
                                Err(e) => {
                                    console::log_1(&format!("验证失败：{:?}", e).into());
                                    is_valid.set(false);
                                }
                            }
                        }
                    }
                    Err(_) => return, // 用户取消了选择
                }
            })
        })
    };

    // --- 3. 渲染部分 ---
    html! {
         <div class="path-card">
            // 标题行：左边是标签，右边是状态
            <div class="path-header">
                <span class="path-label">{"Noita 存档位置 (save00)"}</span>
                {
                    if *is_valid {
                        html! { <span class="badge badge-success">{"● 路径验证通过"}</span> }
                    } else {
                        html! { <span class="badge badge-error">{"● 未找到存档所在"}</span> }
                    }
                }
            </div>

            // 内容行：路径显示 + 修改按钮
            <div class="path-body">
                <div class={if *is_valid { "path-value" } else { "path-value path-error" }}>
                    { &*current_path }
                </div>
                <button onclick={on_select_folder} class="btn btn-secondary btn-browse">
                    {"📁 更改..."}
                </button>
            </div>

            // 错误提示行：仅在无效时显示
            if !*is_valid {
                <div class="path-help-text">
                    {"无法在此路径下检测到存档文件。请手动选择 save00 文件夹"}
                    <br/>
                    {"通常位于: C:/Users/%USERNAME%/AppData/LocalLow/Nolla_Games_Noita/save00"}
                </div>
            }
         </div>
    }
}