use pages::choreography_page::ChoreographyPage;
use pages::dancer_page::DancerPage;
use pages::login_page::LoginPage;
use pages::main_page::MainPage;
use pages::sub_page::info_choreo_page::InfoPage;
use gloo_events::EventListener;
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::window;

use services::supabase::{is_logged_in, logout};

use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
pub mod services;
mod video_thumbnail;

const INACTIVITY_TIMEOUT_MS: u32 = 30 * 60 * 1000;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[at("/login")]
    LoginPage,

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

#[derive(Properties, PartialEq)]
struct AuthenticatedLayoutProps {
    #[prop_or_default]
    children: Children,
}

#[function_component(InactivityLogout)]
fn inactivity_logout() -> Html {
    let navigator = use_navigator();

    use_effect_with((), move |_| {
        let timeout_handle: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));

        let reset_timer = {
            let timeout_handle = timeout_handle.clone();
            let navigator = navigator.clone();

            Rc::new(move || {
                if let Some(old_timeout) = timeout_handle.borrow_mut().take() {
                    old_timeout.cancel();
                }

                let navigator = navigator.clone();

                let timeout = Timeout::new(INACTIVITY_TIMEOUT_MS, move || {
                    let _ = logout();

                    if let Some(nav) = navigator.clone() {
                        nav.push(&Route::LoginPage);
                    }
                });

                *timeout_handle.borrow_mut() = Some(timeout);
            })
        };

        reset_timer();

        let win = window().expect("No browser window found");

        let activity_events = [
            "mousemove",
            "mousedown",
            "keydown",
            "scroll",
            "touchstart",
            "click",
        ];

        let listeners: Vec<EventListener> = activity_events
            .iter()
            .map(|event_name| {
                let reset_timer = reset_timer.clone();

                EventListener::new(&win, *event_name, move |_| {
                    reset_timer();
                })
            })
            .collect();

        move || {
            if let Some(old_timeout) = timeout_handle.borrow_mut().take() {
                old_timeout.cancel();
            }

            drop(listeners);
        }
    });

    html! {}
}

#[function_component(AuthenticatedLayout)]
fn authenticated_layout(props: &AuthenticatedLayoutProps) -> Html {
    let navigator = use_navigator();

    let on_logout = {
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let _ = logout();

            if let Some(nav) = navigator.clone() {
                nav.push(&Route::LoginPage);
            }
        })
    };

    html! {
        <>
            <InactivityLogout />
            <header class="app-header">
                <div class="app-header-title">
                    { "DanceOmatic Creator" }
                </div>

                <nav class="app-header-nav">
                    <Link<Route> to={Route::MainPage}>{ "Home" }</Link<Route>>
                    <Link<Route> to={Route::DancerPage}>{ "Dancers" }</Link<Route>>
                    <Link<Route> to={Route::ChoreographyPage}>{ "Choreographies" }</Link<Route>>

                    <button class="logout-button" onclick={on_logout}>
                        { "Logout" }
                    </button>
                </nav>
            </header>

            <main class="app-content">
                { for props.children.iter() }
            </main>
        </>
    }
}

fn require_login(content: Html) -> Html {
    if is_logged_in() {
        html! {
            <AuthenticatedLayout>
                { content }
            </AuthenticatedLayout>
        }
    } else {
        html! {
            <Redirect<Route> to={Route::LoginPage} />
        }
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::LoginPage => html! { <LoginPage /> },

        Route::MainPage => require_login(html! { <MainPage /> }),

        Route::DancerPage => require_login(html! { <DancerPage /> }),

        Route::ChoreographyPage => require_login(html! { <ChoreographyPage /> }),

        Route::InfoPage { number } => require_login(html! {
            <InfoPage number={number} />
        }),
    }
}