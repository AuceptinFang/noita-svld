use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PathProps {
    pub on_valid_change: Callback<bool>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct SavePathArgs {
    path: String,
}

#[function_component(Path)]
pub fn path(props: &PathProps) -> Html {
    let current_path = use_state(|| "加载中...".to_string());
    let input_ref = use_node_ref();
    let is_valid = use_state(|| false);

    // 初始化时获取路径
    {
        let current_path = current_path.clone();
        let is_valid = is_valid.clone();
        let on_valid_change = props.on_valid_change.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = invoke("get_save_path", JsValue::NULL).await;
                println!("get_save_path response: {:?}", response);

                match response.as_string() {
                    Some(path) => {
                        println!("成功获取路径: {}", path);
                        current_path.set(path);
                        let v = invoke("verify_validation", JsValue::NULL).await;
                        if let Some(_) = v.as_string() {
                            println!("路径验证成功");
                            is_valid.set(true);
                            on_valid_change.emit(true);
                        } else {
                            is_valid.set(false);
                            on_valid_change.emit(false);
                        }
                    }
                    None => {
                        println!("获取路径失败，response不是字符串");
                        current_path.set("获取路径失败".to_string());
                        on_valid_change.emit(false);
                        return;
                    }
                }
            });
            || {}
        });
    }

    // 提交处理
    let on_submit = {
        let current_path = current_path.clone();
        let input_ref = input_ref.clone();
        let is_valid = is_valid.clone();
        let on_valid_change = props.on_valid_change.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let input = input_ref.cast::<web_sys::HtmlInputElement>().unwrap();
            let new_path = input.value();

            if !new_path.is_empty() {
                let current_path = current_path.clone();
                let is_valid = is_valid.clone();
                let on_valid_change = on_valid_change.clone();
                spawn_local(async move {
                    let args = serde_wasm_bindgen::to_value(&SavePathArgs {
                        path: new_path.clone(),
                    })
                    .unwrap();
                    invoke("save_path_to_env", args).await;
                    let v = invoke("verify_validation", JsValue::NULL).await;
                    if let Some(_) = v.as_string() {
                        is_valid.set(true);
                        on_valid_change.emit(true);
                    } else {
                        is_valid.set(false);
                        on_valid_change.emit(false);
                    }
                    current_path.set(new_path);
                });
                input.set_value("");
            }
        })
    };

    let on_select_folder = {
        let current_path = current_path.clone();
        let is_valid = is_valid.clone();
        let on_valid_change = props.on_valid_change.clone();

        Callback::from(move |_: MouseEvent| {
            let current_path = current_path.clone();
            let is_valid = is_valid.clone();
            let on_valid_change = on_valid_change.clone();
            spawn_local(async move {
                let response = invoke("select_save_path", JsValue::NULL).await;
                match response.as_string() {
                    Some(path) => {
                        current_path.set(path.clone());
                        let args =
                            serde_wasm_bindgen::to_value(&SavePathArgs { path: path.clone() })
                                .unwrap();
                        invoke("save_path_to_env", args).await;
                        let v = invoke("verify_validation", JsValue::NULL).await;
                        if let Some(_) = v.as_string() {
                            is_valid.set(true);
                            on_valid_change.emit(true);
                        } else {
                            is_valid.set(false);
                            on_valid_change.emit(false);
                        }
                    }
                    None => return,
                }
            })
        })
    };

    html! {
         <div class="path-container">
             <div class="path-display">
                 <div class="path-info">
                     <span>{ "当前选中路径: " }{&*current_path }</span>
                     {
                         if *is_valid {
                             html! { <span class="path-status-valid">{"已找到存档✓"}</span> }
                         }else {
                             html! { <span class="path-status-invalid">{"路径下未找到存档✗ \n 请手动选择，路径参考：\n C:/Users/%USERNAME%/AppData/LocalLow/Nolla_Games_Noita/save00"}</span> }
                         }
                     }
                 </div>
             </div>
             <div class="class-controls">
                 <form class="path-form" onsubmit={on_submit}>
                     <button onclick={on_select_folder} class="path-button">
                     {"📁 浏览文件"}
                     </button>
                     <input ref={input_ref} placeholder="或直接输入路径（请最好不要）..." class="path-input" />
                     <button type="submit" class="path-submit">
                         {"修改"}
                     </button>
                 </form>
             </div>
         </div>
    }
}
