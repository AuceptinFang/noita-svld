use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wasm_bindgen::prelude::*;
use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct BackupsProps {
    pub is_valid: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: i32,
    pub name: Option<String>,
    pub digest: String,
    pub size: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub save_time: OffsetDateTime,
    pub more_info: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum Operation {
    Save,
    Load,
}

#[function_component(Backups)]
pub fn backups(_props: &BackupsProps) -> Html {
    let current_operation = use_state(|| Operation::Save);
    let backups = use_state(|| Vec::<Backup>::new());
    let show_dialog = use_state(|| false);
    let selected_slot = use_state(|| None::<usize>);

    // 获取所有备份数据
    {
        let backups = backups.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = invoke("get_all_backups", JsValue::NULL).await;
                match serde_wasm_bindgen::from_value::<Vec<Backup>>(response) {
                    Ok(backups_data) => {
                        backups.set(backups_data);
                    }
                    Err(e) => {
                        web_sys::console::log_1(&format!("获取备份失败: {:?}", e).into());
                    }
                }
            });
            || {}
        });
    }

    // 切换到保存模式
    let turn_to_save = {
        let current_operation = current_operation.clone();
        Callback::from(move |_: MouseEvent| {
            current_operation.set(Operation::Save);
        })
    };

    // 切换到加载模式
    let turn_to_load = {
        let current_operation = current_operation.clone();
        Callback::from(move |_: MouseEvent| {
            current_operation.set(Operation::Load);
        })
    };

    // 点击存档槽位
    let on_slot_click = {
        let show_dialog = show_dialog.clone();
        let selected_slot = selected_slot.clone();
        Callback::from(move |slot: usize| {
            selected_slot.set(Some(slot));
            show_dialog.set(true);
        })
    };

    // 关闭弹窗
    let close_dialog = {
        let show_dialog = show_dialog.clone();
        Callback::from(move |_| {
            show_dialog.set(false);
        })
    };

    // 确认操作
    let confirm_operation = {
        let show_dialog = show_dialog.clone();
        let selected_slot = selected_slot.clone();
        let current_operation = current_operation.clone();
        let backups = backups.clone();

        Callback::from(move |_| {
            show_dialog.set(false);

            if let Some(slot) = *selected_slot {
                let operation = (*current_operation).clone();
                let backups = backups.clone();
                spawn_local(async move {
                    // Tauri invoke 需要对象形式的参数，字段名应与后端函数参数 camelCase 一致
                    let (cmd, args_value) = match operation {
                        Operation::Save => (
                            "save_backup",
                            serde_wasm_bindgen::to_value(&json!({ "slotId": slot as i8 })).unwrap()
                        ),
                        Operation::Load => (
                            "load_backup",
                            serde_wasm_bindgen::to_value(&json!({ "backupId": slot as i32 })).unwrap()
                        ),
                    };

                    let response = invoke(cmd, args_value).await;
                    web_sys::console::log_1(&format!("{} 执行结果: {:?}", cmd, response).into());

                    // 重新获取备份列表
                    let response = invoke("get_all_backups", JsValue::NULL).await;
                    if let Ok(backups_data) =
                        serde_wasm_bindgen::from_value::<Vec<Backup>>(response)
                    {
                        backups.set(backups_data);
                    }
                });
            }
        })
    };

    html! {
        <div class="backups-container">
            <div class="backups-wrapper">
                // Save/Load 按钮组
                <div class="backups-buttons">
                    <button
                        class={if *current_operation == Operation::Save { "backup-button backup-save active" } else { "backup-button backup-save" }}
                        onclick={turn_to_save}
                    >
                        {"Save"}
                    </button>
                    <button
                        class={if *current_operation == Operation::Load { "backup-button backup-load active" } else { "backup-button backup-load" }}
                        onclick={turn_to_load}
                    >
                        {"Load"}
                    </button>
                </div>

                // 存档滚动列表
                <div class="backups-list-container">
                    <div class="backups-list">
                    {
                        (0..10).map(|i| {
                            let backup_data = backups.get(i);
                            let slot_number = i + 1;
                            let on_slot_click = on_slot_click.clone();

                            let slot_click = Callback::from(move |_: MouseEvent| {
                                on_slot_click.emit(slot_number);
                            });

                            match backup_data {
                                Some(backup) => {
                                    // 有数据的存档位
                                    let display_name = backup.name.as_ref()
                                        .map(|n| n.clone())
                                        .unwrap_or_else(|| format!("存档位置 {}", slot_number));

                                    let formatted_time = backup.save_time
                                        .format(&time::format_description::well_known::Rfc3339)
                                        .unwrap_or_else(|_| "时间格式错误".to_string());

                                    let size_mb = (backup.size as f64) / (1024.0 * 1024.0);

                                    html! {
                                        <div key={i}
                                             class={if i == 9 { "backup-item backup-item-last" } else { "backup-item" }}
                                             onclick={slot_click}
                                        >
                                            <div class="backup-name">{display_name}</div>
                                            <div class="backup-time">{format!("保存时间: {}", formatted_time)}</div>
                                            <div class="backup-size">{format!("大小: {:.2} MB", size_mb)}</div>
                                            {
                                                if let Some(info) = &backup.more_info {
                                                    html! { <div class="backup-info">{info}</div> }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                    }
                                }
                                None => {
                                    // 空存档位
                                    html! {
                                        <div key={i}
                                             class={if i == 9 { "backup-item backup-item-empty backup-item-last" } else { "backup-item backup-item-empty" }}
                                             onclick={slot_click}
                                        >
                                            <div class="backup-name">{format!("存档位置 {}", slot_number)}</div>
                                            <div class="backup-time">{"空位"}</div>
                                        </div>
                                    }
                                }
                            }
                        }).collect::<Html>()
                    }
                    </div>
                </div>
            </div>

            // 确认弹窗
            {
                if *show_dialog {
                    let (dialog_title, dialog_message, confirm_text) = match *current_operation {
                        Operation::Save => (
                            "保存存档",
                            format!("确认保存到存档位 {} 吗？", selected_slot.unwrap_or(0)),
                            "保存"
                        ),
                        Operation::Load => (
                            "加载存档",
                            format!("确认加载存档位 {} 吗？", selected_slot.unwrap_or(0)),
                            "加载"
                        ),
                    };

                    html! {
                        <div class="modal-overlay" onclick={close_dialog.clone()}>
                            <div class="modal-dialog" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                                <div class="modal-header">
                                    <h3 class="modal-title">{dialog_title}</h3>
                                    <button class="modal-close" onclick={close_dialog.clone()}>{"×"}</button>
                                </div>
                                <div class="modal-body">
                                    <p>{dialog_message}</p>
                                </div>
                                <div class="modal-footer">
                                    <button class="btn btn-secondary" onclick={close_dialog}>{"取消"}</button>
                                    <button class="btn btn-primary" onclick={confirm_operation}>{confirm_text}</button>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
