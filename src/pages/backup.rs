use yew::prelude::*;
use crate::components::{Path,Backups};
#[function_component(Backup)]
pub fn backup() -> Html {
    let is_valid = use_state(|| false);

    let on_valid_change = {
        let is_valid = is_valid.clone();
        Callback::from(move |valid: bool| {
            is_valid.set(valid);
        })
    };

    html! {
        <main class="backup-container">
            // 路径选择组件
            <Path on_valid_change={on_valid_change} />

            <Backups is_valid={*is_valid} />
        </main>
    }
}