use yew::prelude::*;
#[function_component(Info)]
pub fn home() -> Html {
    html! {
        <div class="dashboard-container">
            <h1>{ "有bug请随缘等更新" }</h1>
            <br />
            <h1>{"开发者邮箱：me@aucept.in"}</h1>
        </div>

    }
}