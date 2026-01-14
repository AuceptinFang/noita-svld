use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route; // 确保引入了你的 Route 枚举

#[function_component(Index)]
pub fn home() -> Html {
    // 这里将来可以加载真实的统计数据，比如 invoke("get_stats")
    // 现在先写死或者留空，作为 UI 展示
    let backup_count = 12;
    let total_size = "450 MB";

    html! {
        <div class="dashboard-container">
            // 1. 顶部 Hero 区域
            <div class="hero-section">
                <h1 class="hero-title">{"Noita 存档管理器"}</h1>
            </div>

            // 2. 状态统计卡片 (Dashboard Stats)
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-icon">{"📦"}</div>
                    <div class="stat-info">
                        <span class="stat-value">{backup_count}</span>
                        <span class="stat-label">{"现有存档"}</span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-icon">{"💾"}</div>
                    <div class="stat-info">
                        <span class="stat-value">{total_size}</span>
                        <span class="stat-label">{"占用空间"}</span>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-icon">{"🟢"}</div>
                    <div class="stat-info">
                        <span class="stat-value">{"Ready"}</span>
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