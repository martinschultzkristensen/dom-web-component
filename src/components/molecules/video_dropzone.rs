//components/molecules/video_dropzone.rs
//Purpose of code: presentational single-video dropzone with a "Send to danceOmatic" button,
//used by main_page.rs for the independent "Intro video" / "Load video" uploads.
use crate::video_thumbnail::extract_video_thumbnail;
use web_sys::{DragEvent, Event, File, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct VideoDropzoneProps {
    pub label: String,
    pub video_thumbnail: Option<String>,
    pub on_video_change: Callback<String>,
}

#[function_component(VideoDropzone)]
pub fn video_dropzone(props: &VideoDropzoneProps) -> Html {
    let file_input_ref = use_node_ref();
    let is_dragging_over = use_state(|| false);

    let on_dropzone_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_video_file = {
        let on_video_change = props.on_video_change.clone();
        Callback::from(move |file: File| {
            let on_video_change = on_video_change.clone();
            extract_video_thumbnail(
                file,
                Callback::from(move |data_url: String| on_video_change.emit(data_url)),
            );
        })
    };

    let on_file_change = {
        let on_video_file = on_video_file.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    if let Some(file) = file_list.get(0) {
                        on_video_file.emit(file);
                    }
                }
            }
        })
    };

    let on_dropzone_dragover = {
        let is_dragging_over = is_dragging_over.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            if !*is_dragging_over {
                is_dragging_over.set(true);
            }
        })
    };

    let on_dropzone_dragleave = {
        let is_dragging_over = is_dragging_over.clone();
        Callback::from(move |_: DragEvent| {
            is_dragging_over.set(false);
        })
    };

    let on_dropzone_drop = {
        let on_video_file = on_video_file.clone();
        let is_dragging_over = is_dragging_over.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging_over.set(false);
            if let Some(file) = e
                .data_transfer()
                .and_then(|dt| dt.files())
                .and_then(|files| files.get(0))
            {
                on_video_file.emit(file);
            }
        })
    };

    html! {
        <div class="video-dropzone-panel">
            <h3>{ props.label.clone() }</h3>
            <div
                class="dropzone"
                onclick={on_dropzone_click}
                ondragover={on_dropzone_dragover}
                ondragleave={on_dropzone_dragleave}
                ondrop={on_dropzone_drop}
            >
                if *is_dragging_over {
                    <p class="info-message">{ "Drop video" }</p>
                } else {
                    if let Some(thumbnail) = &props.video_thumbnail {
                        <img src={thumbnail.clone()} alt={props.label.clone()} class="video-thumbnail" />
                    } else {
                        <span>{ format!("Upload {}", props.label) }</span>
                    }
                }
                <input
                    type="file"
                    accept="video/*"
                    ref={file_input_ref}
                    style="display: none;"
                    onchange={on_file_change}
                />
            </div>
            // "Send to danceOmatic" is intentionally disabled: pushing uploads to the
            // danceOmatic server isn't implemented yet.
            <button class="main-action-button" disabled=true>
                { "Send to danceOmatic" }
            </button>
        </div>
    }
}
