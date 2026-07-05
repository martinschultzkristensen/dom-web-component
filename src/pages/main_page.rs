//src/pages/main_page.rs
use yew::prelude::*;
use yew_router::prelude::{Navigator, use_navigator};

#[function_component(MainPage)]
pub fn main_page() -> Html {
    let navigator = use_navigator().unwrap();

    // let press_x_for_main = Callback::from(move |event: KeyboardEvent| {
    //     if event.key() == "x" {
    //         navigator.push(&Route::dancer_page);
    //     }
    // });

    html! {
        <div class="main_menu-container">
            <div class="main-panel">
                <button class="main-action-button">{ "Dancers" }</button>
            </div>
            <div class="main-panel">
                <button class="main-action-button">{ "Choreographies" }</button>
            </div>
        </div>
    }
}
