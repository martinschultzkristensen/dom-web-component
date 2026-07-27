//src/pages/main_page.rs
use yew::prelude::*;
use yew_router::prelude::{use_navigator};
use crate::Route;
use crate::components::molecules::video_dropzone::VideoDropzone;

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

    let show_intro_video = use_state(|| false);
    let show_load_video = use_state(|| false);
    let intro_video_thumbnail = use_state(|| None::<String>);
    let load_video_thumbnail = use_state(|| None::<String>);

    let toggle_intro_video = {
        let show_intro_video = show_intro_video.clone();
        Callback::from(move |_| show_intro_video.set(!*show_intro_video))
    };

    let toggle_load_video = {
        let show_load_video = show_load_video.clone();
        Callback::from(move |_| show_load_video.set(!*show_load_video))
    };

    let on_intro_video_change = {
        let intro_video_thumbnail = intro_video_thumbnail.clone();
        Callback::from(move |data_url: String| intro_video_thumbnail.set(Some(data_url)))
    };

    let on_load_video_change = {
        let load_video_thumbnail = load_video_thumbnail.clone();
        Callback::from(move |data_url: String| load_video_thumbnail.set(Some(data_url)))
    };

     html! {
        <div class="main_menu-container">
            <div class="main-panel">
                <button class="main-action-button" onclick={go_to_dancers}>{ "Dancers" }</button>
            </div>
            <div class="main-panel">
                <button class="main-action-button" onclick={go_to_choreo}>{ "Choreographies" }</button>
            </div>
            <div class="main-panel">
                if *show_intro_video {
                    <VideoDropzone
                        label="Intro video"
                        video_thumbnail={(*intro_video_thumbnail).clone()}
                        on_video_change={on_intro_video_change}
                    />
                } else {
                    <button class="main-action-button" onclick={toggle_intro_video}>{ "Intro video" }</button>
                }
            </div>
            <div class="main-panel">
                if *show_load_video {
                    <VideoDropzone
                        label="Load video"
                        video_thumbnail={(*load_video_thumbnail).clone()}
                        on_video_change={on_load_video_change}
                    />
                } else {
                    <button class="main-action-button" onclick={toggle_load_video}>{ "Load video" }</button>
                }
            </div>
        </div>
    }
}
