use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use wasm_bindgen::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub to: Route,      // 跳转目标
}

#[function_component(SideBar)]
pub fn side_bar(props: &SidebarProps) -> Html {
    let current_route = use_route::<Route>();
    let get_classes = |target: Route| {
        if current_route.as_ref() == Some(&target) {
            "sidebar-item active" // 激活时的样式
        } else {
            "sidebar-item"        // 未激活时的样式
        }
    };

    html! {
        <nav class="sidebar">
            // 首页 (Index)
            <Link<Route> to={Route::Index} classes={get_classes(Route::Index)}>
                { "首页" }
            </Link<Route>>

            // 备份 (Backup)
            <Link<Route> to={Route::Backup} classes={get_classes(Route::Backup)}>
                { "存档" }
            </Link<Route>>

            // 设置 (Settings)
            <Link<Route> to={Route::Settings} classes={get_classes(Route::Settings)}>
                { "设置" }
            </Link<Route>>

            // 信息 (Info)
            <Link<Route> to={Route::Info} classes={get_classes(Route::Info)}>
                { "关于" }
            </Link<Route>>
        </nav>
    }
}