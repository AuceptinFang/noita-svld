use yew::prelude::*;
use crate::components::Version;

#[function_component(Info)]
pub fn home() -> Html {
    html! {
        <div class="dashboard-container">
            <Version />
            <br />
            <h2 class="version-text" style="text-align: center;">{"反馈可发送至开发者邮箱：me@aucept.in"}</h2>
            <br />
        </div>
    }
}