use yew::prelude::*;
use crate::components::Path;
use crate::components::Data;
#[function_component(Setting)]
pub fn home() -> Html {
    html! {
        <div class="dashboard-container">
            <h1>{ "设置" }</h1>
            <Path/>
            <Data/>
        </div>

    }
}