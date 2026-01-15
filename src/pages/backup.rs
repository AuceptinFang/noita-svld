use yew::prelude::*;
use crate::components::{Path,Backups};
#[function_component(Backup_page)]
pub fn backup() -> Html {

    html! {
        <main class="backup-container">
            <Backups/>
        </main>
    }
}