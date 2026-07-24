//components/molecules/video_list.rs
//Purpose of code: presentational list of choreography draft entries used by choreography_page.rs
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{
    CanvasRenderingContext2d, DragEvent, Event, File, HtmlCanvasElement, HtmlInputElement,
    HtmlVideoElement, Url,
};
use yew::prelude::*;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoreographyEntry {
    pub number: u32,
    pub video_thumbnail: Option<String>,
    pub title: String,
    pub duration: String,
}

impl ChoreographyEntry {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            video_thumbnail: None,
            title: String::new(),
            duration: String::new(),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct VideoListProps {
    pub entries: Vec<ChoreographyEntry>,
    pub on_thumbnail_change: Callback<(u32, String)>,
    pub on_title_change: Callback<(u32, String)>,
    pub on_duration_change: Callback<(u32, String)>,
    pub on_checkout: Callback<u32>,
    pub on_add_info: Callback<u32>,
}

#[function_component(VideoList)]
pub fn video_list(props: &VideoListProps) -> Html {
    html! {
        <div class="video-list">
            { for props.entries.iter().map(|entry| html! {
                <VideoListItem
                    key={entry.number}
                    entry={entry.clone()}
                    on_thumbnail_change={props.on_thumbnail_change.clone()}
                    on_title_change={props.on_title_change.clone()}
                    on_duration_change={props.on_duration_change.clone()}
                    on_checkout={props.on_checkout.clone()}
                    on_add_info={props.on_add_info.clone()}
                />
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct VideoListItemProps {
    entry: ChoreographyEntry,
    on_thumbnail_change: Callback<(u32, String)>,
    on_title_change: Callback<(u32, String)>,
    on_duration_change: Callback<(u32, String)>,
    on_checkout: Callback<u32>,
    on_add_info: Callback<u32>,
}

#[function_component(VideoListItem)]
fn video_list_item(props: &VideoListItemProps) -> Html {
    let entry = &props.entry;
    let number = entry.number;
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
        let on_thumbnail_change = props.on_thumbnail_change.clone();
        Callback::from(move |file: File| {
            extract_video_thumbnail(file, number, on_thumbnail_change.clone());
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

    let on_title_input = {
        let on_title_change = props.on_title_change.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                on_title_change.emit((number, input.value()));
            }
        })
    };

    let on_duration_input = {
        let on_duration_change = props.on_duration_change.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                on_duration_change.emit((number, input.value()));
            }
        })
    };

    // Kept for a future "Send til danceOmatic" button; not wired to any button yet.
    let _on_checkout_click = {
        let on_checkout = props.on_checkout.clone();
        Callback::from(move |_: MouseEvent| on_checkout.emit(number))
    };

    let on_add_info_click = {
        let on_add_info = props.on_add_info.clone();
        Callback::from(move |_| on_add_info.emit(number))
    };

    html! {
        <div class="video-list-item">
            <div class="video-list-number">{ format!("NR. {}", number) }</div>

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
                    if let Some(thumbnail) = &entry.video_thumbnail {
                        <img src={thumbnail.clone()} alt="Video thumbnail" class="video-thumbnail" />
                    } else {
                        <span>{ "Upload Demo Video" }</span>
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

            <div class="video-list-fields">
                <input
                    type="text"
                    placeholder="Title:"
                    value={entry.title.clone()}
                    oninput={on_title_input}
                />
                <input
                    type="text"
                    placeholder="Længde:"
                    value={entry.duration.clone()}
                    oninput={on_duration_input}
                />
            </div>

            <button class="main-action-button" onclick={on_add_info_click}>
                { format!("Tilføj info til NR.{}", number) }
            </button>
        </div>
    }
}

// Draws the first frame of the selected video onto an off-screen canvas and
// reports the resulting base64 PNG via `on_thumbnail_change`. Same
// base64-data-URL convention as DancerPage's image upload.
fn extract_video_thumbnail(file: File, number: u32, on_thumbnail_change: Callback<(u32, String)>) {
    let Ok(object_url) = Url::create_object_url_with_blob(&file) else {
        return;
    };
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let Some(video) = document
        .create_element("video")
        .ok()
        .and_then(|el| el.dyn_into::<HtmlVideoElement>().ok())
    else {
        return;
    };

    video.set_src(&object_url);
    video.set_muted(true);
    let _ = video.style().set_property("display", "none");
    let _ = body.append_child(&video);

    let video_for_capture = video.clone();
    let object_url_for_cleanup = object_url.clone();

    let onloadeddata = Closure::wrap(Box::new(move |_event: Event| {
        if let Some(canvas) = document
            .create_element("canvas")
            .ok()
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
        {
            canvas.set_width(video_for_capture.video_width());
            canvas.set_height(video_for_capture.video_height());

            if let Some(ctx) = canvas
                .get_context("2d")
                .ok()
                .flatten()
                .and_then(|ctx| ctx.dyn_into::<CanvasRenderingContext2d>().ok())
            {
                if ctx
                    .draw_image_with_html_video_element(&video_for_capture, 0.0, 0.0)
                    .is_ok()
                {
                    if let Ok(data_url) = canvas.to_data_url() {
                        on_thumbnail_change.emit((number, data_url));
                    }
                }
            }
        }

        video_for_capture.remove();
        let _ = Url::revoke_object_url(&object_url_for_cleanup);
    }) as Box<dyn FnMut(Event)>);

    video.set_onloadeddata(Some(onloadeddata.as_ref().unchecked_ref()));
    onloadeddata.forget();
}
