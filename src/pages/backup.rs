use yew::prelude::*;
use crate::components::{Path,Backups};
#[function_component(Backup)]
pub fn backup() -> Html {

    html! {
        <main class="backups-container">
            <Backups/>
        </main>
    }
}