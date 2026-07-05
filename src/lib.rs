//lib.rs
use pages::main_page::MainPage;
use yew::prelude::*;
use yew_router::prelude::*;

mod pages;
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[at("/")]
    MainPage,
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
    }
}
