//lib.rs
use pages::main_page::MainPage;
use yew::{functional, prelude::*};
use yew_router::prelude::*;

mod pages;
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[at("/main_page")]
    MainPage,
}

#[function_component(DanceOmaticWebComponent)]
pub fn dom_web_component() -> Html {
let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };



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