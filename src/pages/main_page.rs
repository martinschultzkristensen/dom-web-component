//src/pages/main_page.rs
use yew::prelude::*;
use yew_router::prelude::{use_navigator};
use crate::Route;

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
        <div class="main_menu-container">
            <div class="main-panel">
                <button class="main-action-button" onclick={go_to_dancers}>{ "Dancers" }</button>
            </div>
            <div class="main-panel">
                <button class="main-action-button" onclick={go_to_choreo}>{ "Choreographies" }</button>
            </div>
        </div>
    }
}