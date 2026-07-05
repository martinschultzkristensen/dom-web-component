//lib.rs
use components::pages::main_page::MainPage;
use yew::{functional, prelude::*};
use yew_router::prelude::*;

mod components;
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[at("/main_page")]
    MainPage,
}

#[function_component(DanceOmaticWebComponent)]
pub fn dance_o_matic() -> Html {

}


!html {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

fn switch(config: Rc<Config>) -> impl Fn(Route) -> Html {
    move |routes: Route| {
        match routes {
            Route::MainPage => html! { <MainPage /> },
        }
    }
}