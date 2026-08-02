use crate::services::supabase::upload_choreography_file;
use crate::video_thumbnail::extract_video_thumbnail;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{DragEvent, Event, File, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoreographyEntry {
    pub number: u32,

    #[serde(default)]
    pub video_thumbnail: Option<String>,

    #[serde(default)]
    pub demo_video_path: Option<String>,

    #[serde(default)]
    pub title: String,

}

impl ChoreographyEntry {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            video_thumbnail: None,
            demo_video_path: None,
            title: String::new(),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct VideoListProps {
    pub entries: Vec<ChoreographyEntry>,
    pub on_thumbnail_change: Callback<(u32, String)>,
    pub on_demo_video_path_change: Callback<(u32, String)>,
    pub on_title_change: Callback<(u32, String)>,
    pub on_add_info: Callback<u32>,
    pub on_remove: Callback<u32>,
}

#[function_component(VideoList)]
pub fn video_list(props: &VideoListProps) -> Html {
    html! {
        <div class="video-list">
            {
                for props.entries.iter().map(|entry| html! {
                    <VideoListItem
                        key={entry.number}
                        entry={entry.clone()}
                        on_thumbnail_change={props.on_thumbnail_change.clone()}
                        on_demo_video_path_change={props.on_demo_video_path_change.clone()}
                        on_title_change={props.on_title_change.clone()}
                        on_add_info={props.on_add_info.clone()}
                        on_remove={props.on_remove.clone()}
                    />
                })
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct VideoListItemProps {
    entry: ChoreographyEntry,
    on_thumbnail_change: Callback<(u32, String)>,
    on_demo_video_path_change: Callback<(u32, String)>,
    on_title_change: Callback<(u32, String)>,
    on_add_info: Callback<u32>,
    on_remove: Callback<u32>,
}

#[function_component(VideoListItem)]
fn video_list_item(props: &VideoListItemProps) -> Html {
    let entry = &props.entry;
    let number = entry.number;

    let file_input_ref = use_node_ref();
    let is_dragging_over = use_state(|| false);
    let upload_error = use_state(|| None::<String>);
    let is_uploading = use_state(|| false);

    let on_dropzone_click = {
        let file_input_ref = file_input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_video_file = {
        let on_thumbnail_change = props.on_thumbnail_change.clone();
        let on_demo_video_path_change = props.on_demo_video_path_change.clone();
        let upload_error = upload_error.clone();
        let is_uploading = is_uploading.clone();

        Callback::from(move |file: File| {
            let file_for_thumbnail = file.clone();
            let file_for_upload = file;

            let on_thumbnail_change = on_thumbnail_change.clone();
            let on_demo_video_path_change = on_demo_video_path_change.clone();
            let upload_error = upload_error.clone();
            let is_uploading = is_uploading.clone();

            extract_video_thumbnail(
                file_for_thumbnail,
                Callback::from(move |data_url: String| {
                    on_thumbnail_change.emit((number, data_url));
                }),
            );

            is_uploading.set(true);
            upload_error.set(None);

            spawn_local(async move {
                match upload_choreography_file(file_for_upload, "demo_video").await {
                    Ok(path) => {
                        on_demo_video_path_change.emit((number, path));
                        upload_error.set(None);
                    }
                    Err(message) => {
                        upload_error.set(Some(message));
                    }
                }

                is_uploading.set(false);
            });
        })
    };

    let on_file_change = {
        let on_video_file = on_video_file.clone();

        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
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

        Callback::from(move |event: DragEvent| {
            event.prevent_default();

            if !*is_dragging_over {
                is_dragging_over.set(true);
            }
        })
    };

    let on_dropzone_dragleave = {
        let is_dragging_over = is_dragging_over.clone();

        Callback::from(move |_event: DragEvent| {
            is_dragging_over.set(false);
        })
    };

    let on_dropzone_drop = {
        let on_video_file = on_video_file.clone();
        let is_dragging_over = is_dragging_over.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_over.set(false);

            if let Some(file) = event
                .data_transfer()
                .and_then(|data_transfer| data_transfer.files())
                .and_then(|files| files.get(0))
            {
                on_video_file.emit(file);
            }
        })
    };

    let on_title_input = {
        let on_title_change = props.on_title_change.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                on_title_change.emit((number, input.value()));
            }
        })
    };

    let on_add_info_click = {
        let on_add_info = props.on_add_info.clone();

        Callback::from(move |_| {
            on_add_info.emit(number);
        })
    };

    let on_remove_click = {
        let on_remove = props.on_remove.clone();

        Callback::from(move |_| {
            on_remove.emit(number);
        })
    };

    html! {
        <div class="video-list-item">
            <div class="video-list-number">
                { format!("No. {}", number) }
            </div>

            <div
                class="dropzone"
                onclick={on_dropzone_click}
                ondragover={on_dropzone_dragover}
                ondragleave={on_dropzone_dragleave}
                ondrop={on_dropzone_drop}
            >
                if *is_dragging_over {
                    <p class="info-message">{ "Drop video" }</p>
                } else if let Some(thumbnail) = &entry.video_thumbnail {
                    <img
                        src={thumbnail.clone()}
                        alt="Video thumbnail"
                        class="video-thumbnail"
                    />
                } else if *is_uploading {
                    <p class="info-message">{ "Uploading..." }</p>
                } else {
                    <span>{ "Upload Demo Video" }</span>
                }

                if let Some(message) = &*upload_error {
                    <p class="error-message">{ message }</p>
                }

                <input
                    type="file"
                    accept="video/*"
                    ref={file_input_ref}
                    style="display: none;"
                    onchange={on_file_change}
                />
            </div>

            <div class="video-list-fields">
                <input
                    type="text"
                    placeholder="Title:"
                    value={entry.title.clone()}
                    oninput={on_title_input}
                />

            </div>

            <div class="main-panel">
                <button
                    type="button"
                    class="main-action-button"
                    onclick={on_add_info_click}
                >
                    { format!("+ Add details") }
                </button>

                <button
                    type="button"
                    class="choreo-dancer-remove"
                    onclick={on_remove_click}
                >
                    { format!("Remove") }
                </button>
            </div>
        </div>
    }
}