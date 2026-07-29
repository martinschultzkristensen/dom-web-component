use crate::Route;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

#[function_component(MainPage)]
pub fn main_page() -> Html {
    let navigator = use_navigator().unwrap();

    let go_to_dancers = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::DancerPage))
    };

    let go_to_choreo = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::ChoreographyPage))
    };

    html! {
        <div class="main_menu-page">
            <img class="main_menu-hero" src="static/hero.jpg" alt="DanceOmatic" />

            <p class="main_menu-description">
                { "Welcome to DanceOmatic Creator" }
                <br /><br />
                { "Start by creating at least two dancers." }
                <br />
                { "You need a minimum of two dancers before you can create a choreography." }
                <br /><br />
                { "After your dancers are ready, you can continue to the choreography page and upload your choreography content." }
            </p>

            <div class="main_menu-container">
                <div class="main-panel">
                    <button class="main-action-button" onclick={go_to_dancers}>
                        { "Dancers" }
                    </button>
                </div>

                <div class="main-panel">
                    <button class="main-action-button" onclick={go_to_choreo}>
                        { "Choreographies" }
                    </button>
                </div>
            </div>
        </div>
    }
}