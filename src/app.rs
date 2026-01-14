use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::*;
use crate::pages::{index::Index, backup::Backup,setting::Setting};
use crate::router::Route;
use crate::components::SideBar;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

fn switch(routes: Route) -> Html {
    match routes {
        // 当路由器发现路径是 "/" (Route::Index) 时
        // 它执行下面的代码，把 <Index /> 组件画出来
        Route::Index => html! { <Index /> },

        Route::Backup => html! { <Backup /> },

        Route::Info => html! { <h1>{ "开发信息" }</h1> },

        Route::Settings => html! { <Setting /> },

        Route::NotFound => html! { <h1>{ "404 页面不存在" }</h1> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="app-container">
                <SideBar
                    to={Route::Index}
                />
                <main class="main-container">
                    <Switch<Route> render={switch} />
                </main>
            </div>
        </BrowserRouter>
    }
}

