use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::Backup;
use crate::router::Route; // 确保引入了你的 Route 枚举

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct DashboardStats {
    backup_count: usize,
    total_size: u64,    // 单位字节
    is_ready: bool,     // 后端健康
}

impl DashboardStats {
    pub fn formatted_size(&self) -> String {
        let size = self.total_size as f64;
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        if size < KB {
            format!("{} B", size)
        } else if size < MB {
            format!("{:.1} KB", size / KB) // 保留1位小数
        } else if size < GB {
            format!("{:.1} MB", size / MB)
        } else {
            format!("{:.2} GB", size / GB) // 保留2位小数
        }
    }
}

#[function_component(Index)]
pub fn home() -> Html {
    let stats = use_state(|| DashboardStats {
        backup_count: 0,
        total_size: u64::MAX,
        is_ready: false,
    });

    {
        let stats = stats.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                // 1. 直接获取 JsValue，不进行 Result 匹配
                let response = invoke("get_dashboard_stats", JsValue::NULL).await;

                // 2. 在反序列化 (from_value) 时进行 match
                match serde_wasm_bindgen::from_value::<DashboardStats>(response) {
                    Ok(fetched_stats) => {
                        stats.set(fetched_stats);
                    },
                    Err(e) => {
                        // 记录反序列化错误，比如后端没返回数据
                        web_sys::console::log_1(&format!("无法解析统计数据: {:?}", e).into());
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="dashboard-container">
            <div class="hero-section">
                <h1 class="hero-title">{"Noita 存档管理器"}</h1>
            </div>

            // 状态卡片 (Dashboard Stats)
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-icon">{"📦"}</div>
                    <div class="stat-info">
                        <span class="stat-value">{stats.backup_count}</span>
                        <span class="stat-label">{"现有存档"}</span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-icon">{"💾"}</div>
                    <div class="stat-info">
                        <span class="stat-value">{stats.formatted_size()}</span>
                        <span class="stat-label">{"占用空间"}</span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-icon">
                        if stats.is_ready { {"🟢"} } else { {"🟠"} }
                    </div>
                    <div class="stat-info">
                        <span class="stat-value">
                            if stats.is_ready { {"Ready"} } else { {"Connecting..."} }
                        </span>
                        <span class="stat-label">{"后端状态"}</span>
                    </div>
                </div>
            </div>

            // 3. 快速导航入口
            <div class="actions-grid">
                <Link<Route> to={Route::Backup} classes="action-card action-primary">
                    <div class="action-content">
                        <span class="action-icon">{"⚡"}</span>
                        <h3>{"管理存档"}</h3>
                        <p>{"保存、加载存档"}</p>
                    </div>
                    <div class="action-arrow">{"→"}</div>
                </Link<Route>>

                <Link<Route> to={Route::Settings} classes="action-card action-secondary">
                    <div class="action-content">
                        <span class="action-icon">{"⚙️"}</span>
                        <h3>{"设置"}</h3>
                        <p>{"请确保游戏路径选择正确"}</p>
                    </div>
                    <div class="action-arrow">{"→"}</div>
                </Link<Route>>
            </div>

            // 4. 底部装饰或提示
            <div class="footer-tip">
                {"💡 tips ：请向哈米斯投喂石板，它会报答你的"}
            </div>
        </div>
    }
}