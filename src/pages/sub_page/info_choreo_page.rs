use crate::components::molecules::video_list::ChoreographyEntry;
use crate::pages::choreography_page::DRAFT_CHOREOGRAPHIES_STORAGE_KEY;
use crate::services::supabase::{fetch_dancers, DancerRow};
use crate::video_thumbnail::extract_video_thumbnail;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    DragEvent, Event, File, FileReader, HtmlInputElement, HtmlTextAreaElement,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InfoPageProps {
    pub number: u32,
}

#[derive(Clone, PartialEq)]
struct DancerOption {
    id: String,
    name: String,
}

fn dancer_rows_to_options(rows: Vec<DancerRow>) -> Vec<DancerOption> {
    rows.into_iter()
        .map(|dancer| DancerOption {
            id: dancer.id,
            name: dancer.name,
        })
        .collect()
}

fn load_draft_entry(number: u32) -> Option<ChoreographyEntry> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| {
            storage
                .get_item(DRAFT_CHOREOGRAPHIES_STORAGE_KEY)
                .ok()
                .flatten()
        })
        .and_then(|json| serde_json::from_str::<Vec<ChoreographyEntry>>(&json).ok())
        .and_then(|entries| entries.into_iter().find(|entry| entry.number == number))
}

fn load_title(number: u32) -> String {
    load_draft_entry(number)
        .map(|entry| entry.title)
        .unwrap_or_default()
}

fn load_target_machine_label(number: u32) -> String {
    load_draft_entry(number)
        .map(|entry| match entry.target_machine.as_str() {
            "machine_1" => "DanceOmatic 1".to_string(),
            "machine_2" => "DanceOmatic 2".to_string(),
            "machine_3" => "DanceOmatic 3".to_string(),
            _ => "DanceOmatic".to_string(),
        })
        .unwrap_or_else(|| "DanceOmatic".to_string())
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
struct ChoreographyInfo {
    #[serde(default)]
    choreo_image: Option<String>,

    #[serde(default)]
    choreo_video_thumbnail: Option<String>,

    #[serde(default)]
    description: String,

    #[serde(default)]
    dancer_ids: Vec<String>,
}

fn non_empty(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect()
}

fn storage_key(number: u32) -> String {
    format!("choreo_info_{number}")
}

fn load_choreography_info(number: u32) -> ChoreographyInfo {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(&storage_key(number)).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_choreography_info(number: u32, info: &ChoreographyInfo) {
    if let Ok(json) = serde_json::to_string(info) {
        if let Some(storage) =
            web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item(&storage_key(number), &json);
        }
    }
}

fn show_alert(message: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.alert_with_message(message);
    }
}

fn validate_choreography_for_send(
    number: u32,
    choreo_image: &Option<String>,
    description: &str,
    choreo_video_thumbnail: &Option<String>,
    selected_dancer_ids: &[String],
) -> Vec<&'static str> {
    let mut missing_fields = Vec::new();

    match load_draft_entry(number) {
        Some(entry) => {
            if entry.title.trim().is_empty() {
                missing_fields.push("Title");
            }

            if entry.duration.trim().is_empty() {
                missing_fields.push("Duration");
            }

            if entry.video_thumbnail.is_none() {
                missing_fields.push("Demo video");
            }

            if entry.target_machine.trim().is_empty() {
                missing_fields.push("DanceOmatic machine");
            }
        }
        None => {
            missing_fields.push("Choreography draft");
        }
    }

    if choreo_image.is_none() {
        missing_fields.push("Choreography image");
    }

    if description.trim().is_empty() {
        missing_fields.push("Description");
    }

    if choreo_video_thumbnail.is_none() {
        missing_fields.push("Choreography video");
    }

    if non_empty(selected_dancer_ids).len() < 2 {
        missing_fields.push("At least two dancers");
    }

    missing_fields
}

#[function_component(InfoPage)]
pub fn info_page(props: &InfoPageProps) -> Html {
    let number = props.number;
    let saved_info = load_choreography_info(number);

    let initial_choreo_image = saved_info.choreo_image.clone();
    let initial_choreo_video_thumbnail = saved_info.choreo_video_thumbnail.clone();
    let initial_description = saved_info.description.clone();
    let initial_dancer_ids = non_empty(&saved_info.dancer_ids);

    let title = use_memo(number, |number| load_title(*number));
    let target_machine_label = use_memo(number, |number| load_target_machine_label(*number));

    let choreo_image = use_state(move || initial_choreo_image);
    let choreo_video_thumbnail = use_state(move || initial_choreo_video_thumbnail);
    let description = use_state(move || initial_description);
    let selected_dancer_ids = use_state(move || initial_dancer_ids);

    let all_dancers = use_state(Vec::<DancerOption>::new);
    let dancers_error = use_state(|| None::<String>);
    let is_dancer_picker_open = use_state(|| false);

    let is_dragging_over_image = use_state(|| false);
    let is_dragging_over_video = use_state(|| false);
    let image_input_ref = use_node_ref();
    let video_input_ref = use_node_ref();

    {
        let all_dancers = all_dancers.clone();
        let dancers_error = dancers_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match fetch_dancers().await {
                    Ok(rows) => {
                        all_dancers.set(dancer_rows_to_options(rows));
                        dancers_error.set(None);
                    }
                    Err(message) => {
                        all_dancers.set(Vec::new());
                        dancers_error.set(Some(message));
                    }
                }
            });

            || ()
        });
    }

    {
        let choreo_image = choreo_image.clone();
        let choreo_video_thumbnail = choreo_video_thumbnail.clone();
        let description = description.clone();
        let selected_dancer_ids = selected_dancer_ids.clone();

        use_effect_with(
            (
                (*choreo_image).clone(),
                (*choreo_video_thumbnail).clone(),
                (*description).clone(),
                (*selected_dancer_ids).clone(),
            ),
            move |_| {
                save_choreography_info(
                    number,
                    &ChoreographyInfo {
                        choreo_image: (*choreo_image).clone(),
                        choreo_video_thumbnail: (*choreo_video_thumbnail).clone(),
                        description: (*description).clone(),
                        dancer_ids: non_empty(&*selected_dancer_ids),
                    },
                );

                || ()
            },
        );
    }

    let on_image_dropzone_click = {
        let image_input_ref = image_input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = image_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_image_file = {
        let choreo_image = choreo_image.clone();

        Callback::from(move |file: File| {
            let Ok(reader) = FileReader::new() else {
                return;
            };

            let reader_clone = reader.clone();
            let choreo_image = choreo_image.clone();

            let onload = Closure::wrap(Box::new(move |_event: Event| {
                if let Ok(result) = reader_clone.result() {
                    if let Some(data_url) = result.as_string() {
                        choreo_image.set(Some(data_url));
                    }
                }
            }) as Box<dyn FnMut(Event)>);

            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            onload.forget();

            let _ = reader.read_as_data_url(&file);
        })
    };

    let on_image_file_change = {
        let on_image_file = on_image_file.clone();

        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        on_image_file.emit(file);
                    }
                }
            }
        })
    };

    let on_image_dropzone_dragover = {
        let is_dragging_over_image = is_dragging_over_image.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_over_image.set(true);
        })
    };

    let on_image_dropzone_dragleave = {
        let is_dragging_over_image = is_dragging_over_image.clone();

        Callback::from(move |_event: DragEvent| {
            is_dragging_over_image.set(false);
        })
    };

    let on_image_dropzone_drop = {
        let on_image_file = on_image_file.clone();
        let is_dragging_over_image = is_dragging_over_image.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_over_image.set(false);

            if let Some(file) = event
                .data_transfer()
                .and_then(|data_transfer| data_transfer.files())
                .and_then(|files| files.get(0))
            {
                on_image_file.emit(file);
            }
        })
    };

    let on_video_dropzone_click = {
        let video_input_ref = video_input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = video_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_video_file = {
        let choreo_video_thumbnail = choreo_video_thumbnail.clone();

        Callback::from(move |file: File| {
            let choreo_video_thumbnail = choreo_video_thumbnail.clone();

            extract_video_thumbnail(
                file,
                Callback::from(move |data_url: String| {
                    choreo_video_thumbnail.set(Some(data_url));
                }),
            );
        })
    };

    let on_video_file_change = {
        let on_video_file = on_video_file.clone();

        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        on_video_file.emit(file);
                    }
                }
            }
        })
    };

    let on_video_dropzone_dragover = {
        let is_dragging_over_video = is_dragging_over_video.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_over_video.set(true);
        })
    };

    let on_video_dropzone_dragleave = {
        let is_dragging_over_video = is_dragging_over_video.clone();

        Callback::from(move |_event: DragEvent| {
            is_dragging_over_video.set(false);
        })
    };

    let on_video_dropzone_drop = {
        let on_video_file = on_video_file.clone();
        let is_dragging_over_video = is_dragging_over_video.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_over_video.set(false);

            if let Some(file) = event
                .data_transfer()
                .and_then(|data_transfer| data_transfer.files())
                .and_then(|files| files.get(0))
            {
                on_video_file.emit(file);
            }
        })
    };

    let on_description_input = {
        let description = description.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(textarea) = event.target_dyn_into::<HtmlTextAreaElement>() {
                description.set(textarea.value());
            }
        })
    };

    let on_open_dancer_picker = {
        let is_dancer_picker_open = is_dancer_picker_open.clone();

        Callback::from(move |_| {
            is_dancer_picker_open.set(true);
        })
    };

    let on_close_dancer_picker = {
        let is_dancer_picker_open = is_dancer_picker_open.clone();

        Callback::from(move |_| {
            is_dancer_picker_open.set(false);
        })
    };

    let on_add_dancer = {
        let selected_dancer_ids = selected_dancer_ids.clone();
        let is_dancer_picker_open = is_dancer_picker_open.clone();

        Callback::from(move |dancer_id: String| {
            let mut updated = (*selected_dancer_ids).clone();

            if !updated.contains(&dancer_id) {
                updated.push(dancer_id);
            }

            selected_dancer_ids.set(non_empty(&updated));
            is_dancer_picker_open.set(false);
        })
    };

    let on_dancer_remove = {
        let selected_dancer_ids = selected_dancer_ids.clone();

        Callback::from(move |dancer_id_to_remove: String| {
            let updated = (*selected_dancer_ids)
                .clone()
                .into_iter()
                .filter(|dancer_id| dancer_id != &dancer_id_to_remove)
                .collect::<Vec<String>>();

            selected_dancer_ids.set(updated);
        })
    };

    let on_send_click = {
        let choreo_image = choreo_image.clone();
        let choreo_video_thumbnail = choreo_video_thumbnail.clone();
        let description = description.clone();
        let selected_dancer_ids = selected_dancer_ids.clone();

        Callback::from(move |_| {
            let description_value = (*description).clone();
            let selected_ids = (*selected_dancer_ids).clone();

            let missing_fields = validate_choreography_for_send(
                number,
                &*choreo_image,
                &description_value,
                &*choreo_video_thumbnail,
                &selected_ids,
            );

            if !missing_fields.is_empty() {
                let message = format!(
                    "Please complete the following required fields:\n\n{}",
                    missing_fields
                        .iter()
                        .map(|field| format!("- {}", field))
                        .collect::<Vec<String>>()
                        .join("\n")
                );

                show_alert(&message);
                return;
            }

            show_alert(
                "Ready to send. Supabase upload and database submit will be connected in the next step.",
            );
        })
    };

    let selected_ids = (*selected_dancer_ids).clone();

    let available_dancers = all_dancers
        .iter()
        .filter(|dancer| !selected_ids.contains(&dancer.id))
        .cloned()
        .collect::<Vec<DancerOption>>();

    html! {
        <div class="page about-choreo-container">
            <div class="arcadefont">
                <h2>{ format!("Choreography Details No. {}", number) }</h2>

                <div class="creator-help-box">
                    <p>
                        { "Complete the choreography image, description, choreography video and dancers before sending." }
                    </p>
                    <p>
                        { "At least two dancers must be selected. Click Add Dancer and choose dancers from the list." }
                    </p>
                </div>

                <input
                    type="text"
                    class="choreo-title-display"
                    placeholder="Title is copied from the choreography draft"
                    value={(*title).clone()}
                    readonly=true
                />

                <div class="choreo-info-section">
                    <div
                        class="choreo-image-dropzone"
                        onclick={on_image_dropzone_click}
                        ondragover={on_image_dropzone_dragover}
                        ondragleave={on_image_dropzone_dragleave}
                        ondrop={on_image_dropzone_drop}
                    >
                        if *is_dragging_over_image {
                            <p class="info-message">{ "Drop image" }</p>
                        } else if let Some(image) = &*choreo_image {
                            <img src={image.clone()} alt="Choreography" />
                        } else {
                            <span>{ "Upload Image" }</span>
                        }

                        <input
                            type="file"
                            accept="image/*"
                            ref={image_input_ref}
                            style="display: none;"
                            onchange={on_image_file_change}
                        />
                    </div>

                    <textarea
                        class="choreo-description-input"
                        placeholder="Describe the choreography:"
                        value={(*description).clone()}
                        oninput={on_description_input}
                    />
                </div>

                <h2>{ "Choreography Video" }</h2>

                <div
                    class="dropzone"
                    onclick={on_video_dropzone_click}
                    ondragover={on_video_dropzone_dragover}
                    ondragleave={on_video_dropzone_dragleave}
                    ondrop={on_video_dropzone_drop}
                >
                    if *is_dragging_over_video {
                        <p class="info-message">{ "Drop video" }</p>
                    } else if let Some(thumbnail) = &*choreo_video_thumbnail {
                        <img
                            src={thumbnail.clone()}
                            alt="Choreography video thumbnail"
                            class="video-thumbnail"
                        />
                    } else {
                        <span>{ "Upload Choreography Video" }</span>
                    }

                    <input
                        type="file"
                        accept="video/*"
                        ref={video_input_ref}
                        style="display: none;"
                        onchange={on_video_file_change}
                    />
                </div>

                <h2>{ "Dancers" }</h2>

                if let Some(message) = &*dancers_error {
                    <p class="error-message">{ message }</p>
                }

                <div class="choreo-dancers-section">
                    <button
                        type="button"
                        class="add-dancer-button"
                        onclick={on_open_dancer_picker}
                    >
                        { "Add Dancer" }
                    </button>

                    if *is_dancer_picker_open {
                        <div class="dancer-picker-panel">
                            <div class="dancer-picker-header">
                                <span>{ "Choose dancer" }</span>

                                <button
                                    type="button"
                                    class="small-remove-button"
                                    onclick={on_close_dancer_picker}
                                >
                                    { "Cancel" }
                                </button>
                            </div>

                            if available_dancers.is_empty() {
                                <p class="login-help-text">
                                    { "All available dancers have already been selected." }
                                </p>
                            } else {
                                <div class="dancer-picker-options">
                                    {
                                        for available_dancers.iter().cloned().map(|dancer| {
                                            let on_add_dancer = on_add_dancer.clone();
                                            let dancer_id = dancer.id.clone();

                                            html! {
                                                <button
                                                    key={dancer.id.clone()}
                                                    type="button"
                                                    class="dancer-picker-option"
                                                    onclick={Callback::from(move |_| {
                                                        on_add_dancer.emit(dancer_id.clone());
                                                    })}
                                                >
                                                    { dancer.name.clone() }
                                                </button>
                                            }
                                        })
                                    }
                                </div>
                            }
                        </div>
                    }

                    if !selected_ids.is_empty() {
                        <div class="selected-dancers-list">
                            {
                                for selected_ids.iter().cloned().map(|selected_id| {
                                    let dancer_name = all_dancers
                                        .iter()
                                        .find(|dancer| dancer.id == selected_id)
                                        .map(|dancer| dancer.name.clone())
                                        .unwrap_or_else(|| "Unknown dancer".to_string());

                                    let on_remove_click = {
                                        let on_dancer_remove = on_dancer_remove.clone();
                                        let selected_id_for_remove = selected_id.clone();

                                        Callback::from(move |_| {
                                            on_dancer_remove.emit(selected_id_for_remove.clone());
                                        })
                                    };

                                    html! {
                                        <div key={selected_id.clone()} class="selected-dancer-card">
                                            <div class="selected-dancer-name">
                                                { dancer_name }
                                            </div>

                                            <button
                                                type="button"
                                                class="small-remove-button"
                                                onclick={on_remove_click}
                                            >
                                                { "Remove" }
                                            </button>
                                        </div>
                                    }
                                })
                            }
                        </div>
                    }
                </div>

                <div class="submit-choreography-panel">
                   <button
                        type="button"
                        class="send-choreography-button"
                        onclick={on_send_click}
                    >
                        { format!("Send to {}", (*target_machine_label).clone()) }
                    </button>
                </div>
            </div>
        </div>
    }
}