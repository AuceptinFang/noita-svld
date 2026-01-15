use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wasm_bindgen::prelude::*;
use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
// 引入你的 Path 组件
use crate::components::Path;

#[derive(Properties, PartialEq, Clone)]
pub struct BackupsProps {
    pub is_valid: bool,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

// 对应后端的数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Backup {
    pub id: i32, // 唯一标识
    pub name: Option<String>, // 备注
    pub size: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub save_time: OffsetDateTime,
}

// 用于控制弹窗状态的枚举
#[derive(Clone, PartialEq)]
enum ModalAction {
    None,
    ConfirmRestore(i32, String), // id, name
    ConfirmDelete(i32, String),  // id, name
}

#[function_component(Backups)]
pub fn backups() -> Html {
    let backups_list = use_state(|| Vec::<Backup>::new());
    let note_input_ref = use_node_ref();
    let modal_state = use_state(|| ModalAction::None);

    // 获取备份列表
    let fetch_backups = {
        let backups_list = backups_list.clone();
        move || {
            let backups_list = backups_list.clone();
            spawn_local(async move {
                let response = invoke("get_all_backups", JsValue::NULL).await;
                match serde_wasm_bindgen::from_value::<Vec<Backup>>(response) {
                    Ok(mut data) => {
                        // 按时间倒序排序（最新的在最上面）
                        data.sort_by(|a, b| b.save_time.cmp(&a.save_time));
                        backups_list.set(data);
                    }
                    Err(e) => web_sys::console::log_1(&format!("Err: {:?}", e).into()),
                }
            });
        }
    };

    // 初始化加载
    {
        let fetch = fetch_backups.clone();
        use_effect_with((), move |_| {
            fetch();
            || {}
        });
    }

    // 创建备份 (Create)
    let on_create_click = {
        let note_input_ref = note_input_ref.clone();
        let fetch = fetch_backups.clone();

        Callback::from(move |e: MouseEvent| {
            e.prevent_default(); // 防止Form提交刷新
            let input = note_input_ref.cast::<web_sys::HtmlInputElement>().unwrap();
            let note = input.value();

            let fetch = fetch.clone();
            let input_clone = input.clone();

            spawn_local(async move {
                // 调用 Tauri: create_backup
                let args = serde_wasm_bindgen::to_value(&json!({ "note": note })).unwrap();
                invoke("create_backup", args).await;

                // 清空输入框并刷新列表
                input_clone.set_value("");
                fetch();
            });
        })
    };

    // 触发弹窗逻辑
    let trigger_restore = {
        let modal_state = modal_state.clone();
        Callback::from(move |(id, name): (i32, String)| {
            modal_state.set(ModalAction::ConfirmRestore(id, name));
        })
    };

    let trigger_delete = {
        let modal_state = modal_state.clone();
        Callback::from(move |(id, name): (i32, String)| {
            modal_state.set(ModalAction::ConfirmDelete(id, name));
        })
    };

    // 执行确认操作 (Modal Confirm)
    let on_modal_confirm = {
        let modal_state = modal_state.clone();
        let fetch = fetch_backups.clone();

        Callback::from(move |_| {
            let fetch = fetch.clone();
            let current_action = (*modal_state).clone();

            spawn_local(async move {
                match current_action {
                    ModalAction::ConfirmRestore(id, _) => {
                        let args = serde_wasm_bindgen::to_value(&json!({ "id": id })).unwrap();
                        invoke("restore_backup", args).await;
                        // 还原后可能不需要刷新列表，但为了保险起见可以刷新
                    },
                    ModalAction::ConfirmDelete(id, _) => {
                        let args = serde_wasm_bindgen::to_value(&json!({ "id": id })).unwrap();
                        invoke("delete_backup", args).await;
                        fetch(); // 删除后必须刷新列表
                    },
                    ModalAction::None => {}
                }
            });
            modal_state.set(ModalAction::None); // 关闭弹窗
        })
    };

    let on_modal_cancel = {
        let modal_state = modal_state.clone();
        Callback::from(move |_| modal_state.set(ModalAction::None))
    };

    // --- 渲染 ---
    html! {
        <div class="flex-col w-full h-full"> //新建备份区域

            <div class="backup-maker">
                <input
                    ref={note_input_ref}
                    class="backup-note-input"
                    type="text"
                    placeholder="添加备注"
                />
                <button class="btn btn-create btn-primary" onclick={on_create_click}>
                    <span>{"Save"}</span>
                </button>
            </div>


            // B. 备份列表区域
            <div class="backup-list-container mt-4">
                if backups_list.is_empty() {
                     <div class="backup-card">
                        // 左侧信息
                        <div class="card-info">
                            <h4>{ "暂无备份记录" }</h4>
                            <div class="card-meta">
                            </div>
                        </div>
                    </div>
                } else {
                    { for backups_list.iter().map(|backup| {
                        let id = backup.id;
                        let name = backup.name.clone().unwrap_or_else(|| "未命名备份".to_string());
                        let name_for_restore = name.clone();
                        let name_for_delete = name.clone();

                        let size_mb = (backup.size as f64) / (1024.0 * 1024.0);
                        // 简单格式化时间
                        let time_str = backup.save_time.format(&time::format_description::well_known::Rfc3339).unwrap_or("Unknown".into());
                        // 实际项目中建议用 time crate 自定义 format_description 来显示更友好的 "YYYY-MM-DD HH:MM"

                        let on_restore = trigger_restore.clone();
                        let on_delete = trigger_delete.clone();

                        html! {
                            <div class="backup-card">
                                // 左侧信息
                                <div class="card-info">
                                    <h4>{ &name }</h4>
                                    <div class="card-meta">
                                        <span>{ "📅 " }{ &time_str }</span>
                                        <span>{ "💿 " }{ format!("{:.2} MB", size_mb) }</span>
                                    </div>
                                </div>

                                // 右侧操作按钮
                                <div class="card-actions">
                                    <button
                                        class="btn btn-restore"
                                        onclick={Callback::from(move |_| on_restore.emit((id, name_for_restore.clone())))}
                                    >
                                        {"Load"}
                                    </button>
                                    <button
                                        class="btn btn-delete"
                                        onclick={Callback::from(move |_| on_delete.emit((id, name_for_delete.clone())))}
                                        title="删除此备份"
                                    >
                                        {"Delete"}
                                    </button>
                                </div>
                            </div>
                        }
                    })}
                }
            </div>

            // C. 弹窗组件
            if *modal_state != ModalAction::None {
                <div class="modal-overlay" onclick={on_modal_cancel.clone()}>
                    <div class="modal-dialog" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                        <div class="modal-header">
                            <h3 class="modal-title">
                                {match *modal_state {
                                    ModalAction::ConfirmRestore(_, _) => "确认还原存档？",
                                    ModalAction::ConfirmDelete(_, _) => "确认删除备份？",
                                    _ => ""
                                }}
                            </h3>
                        </div>
                        <div class="modal-body py-4 text-slate-300">
                            {match &*modal_state {
                                ModalAction::ConfirmRestore(_, name) => format!("确定要回退到 [{}] 吗？\n当前的游戏进度将会被覆盖且无法找回！", name),
                                ModalAction::ConfirmDelete(_, name) => format!("确定要永久删除 [{}] 吗？此操作无法撤销。", name),
                                _ => "".to_string()
                            }}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-secondary" onclick={on_modal_cancel}>{"取消"}</button>
                            <button class="btn btn-primary" onclick={on_modal_confirm}>{"确定"}</button>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}