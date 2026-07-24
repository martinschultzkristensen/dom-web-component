//lib.rs
use pages::choreography_page::ChoreographyPage;
use pages::dancer_page::DancerPage;
use pages::main_page::MainPage;
use pages::sub_page::info_choreo_page::InfoPage;

use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[at("/")]
    MainPage,
    #[at("/dancers")]
    DancerPage,
    #[at("/choreographies")]
    ChoreographyPage,
    #[at("/choreographies/:number/info")]
    InfoPage { number: u32 },
}

#[function_component(DanceOmaticWebComponent)]
pub fn dom_web_component() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Callback::from(switch)} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::MainPage => html! { <MainPage /> },
        Route::DancerPage => html! { <DancerPage /> },
        Route::ChoreographyPage => html! { <ChoreographyPage /> },
        Route::InfoPage { number } => html! { <InfoPage number={number} /> },
    }
}
