use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use wasm_bindgen::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SidebarItemProps {
    pub to: Route,      // 跳转目标
    pub label: String,  // 显示文字
}

#[function_component(SideBar)]
pub fn side_bar(props: &SidebarItemProps) -> Html {
    let current_route = use_route::<Route>();
    html!{
        
    }
}