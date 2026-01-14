use yew_router::prelude::*;

#[derive(Clone,Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Index,
    #[at("/backup")]
    Backup,
    #[at("/info")]
    Info,
    #[at("/settings")]
    Settings,
    #[not_found]
    #[at("/404")]
    NotFound,
}