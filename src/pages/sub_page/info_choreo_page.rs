//src/pages/sub_page/info_choreo_page.rs
use crate::components::molecules::video_list::ChoreographyEntry;
use crate::pages::choreography_page::DRAFT_CHOREOGRAPHIES_STORAGE_KEY;
use crate::pages::dancer_page::load_dancers;
use crate::video_thumbnail::extract_video_thumbnail;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{
    DragEvent, Event, File, FileReader, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InfoPageProps {
    pub number: u32,
}

// Reads the title copied over from the matching draft entry on choreography_page.rs
fn load_title(number: u32) -> String {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| {
            storage
                .get_item(DRAFT_CHOREOGRAPHIES_STORAGE_KEY)
                .ok()
                .flatten()
        })
        .and_then(|json| serde_json::from_str::<Vec<ChoreographyEntry>>(&json).ok())
        .and_then(|entries| entries.into_iter().find(|entry| entry.number == number))
        .map(|entry| entry.title)
        .unwrap_or_default()
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
struct ChoreographyInfo {
    choreo_image: Option<String>,
    choreo_video_thumbnail: Option<String>,
    description: String,
    dancers: Vec<String>,
}

// Keeps a trailing empty slot so the dancers list always has one free
// dropdown to pick another dancer from.
fn with_trailing_empty_slot(mut names: Vec<String>) -> Vec<String> {
    if names.last().is_none_or(|name| !name.is_empty()) {
        names.push(String::new());
    }
    names
}

fn non_empty(names: &[String]) -> Vec<String> {
    names.iter().filter(|name| !name.is_empty()).cloned().collect()
}

fn storage_key(number: u32) -> String {
    format!("choreo_info_{number}")
}

fn load_choreography_info(number: u32) -> ChoreographyInfo {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(&storage_key(number)).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_choreography_info(number: u32, info: &ChoreographyInfo) {
    if let Ok(json) = serde_json::to_string(info) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(&storage_key(number), &json);
        }
    }
}

#[function_component(InfoPage)]
pub fn info_page(props: &InfoPageProps) -> Html {
    let number = props.number;
    let title = use_memo(number, |number| load_title(*number));

    // Choreo image + video thumbnail + description + dancers are persisted to localStorage
    // (per choreography number) so edits survive a refresh, same convention as choreography_page.rs.
    let choreo_image = use_state(|| load_choreography_info(number).choreo_image);
    let choreo_video_thumbnail =
        use_state(|| load_choreography_info(number).choreo_video_thumbnail);
    let description = use_state(|| load_choreography_info(number).description);
    let selected_dancers = use_state(|| with_trailing_empty_slot(load_choreography_info(number).dancers));
    let all_dancers = use_state(load_dancers);
    let is_dragging_over_image = use_state(|| false);
    let is_dragging_over_video = use_state(|| false);
    let image_input_ref = use_node_ref();
    let video_input_ref = use_node_ref();

    {
        let choreo_video_thumbnail = choreo_video_thumbnail.clone();
        let description = description.clone();
        let selected_dancers = selected_dancers.clone();
        use_effect_with(choreo_image.clone(), move |choreo_image| {
            save_choreography_info(
                number,
                &ChoreographyInfo {
                    choreo_image: (**choreo_image).clone(),
                    choreo_video_thumbnail: (*choreo_video_thumbnail).clone(),
                    description: (*description).clone(),
                    dancers: non_empty(&selected_dancers),
                },
            );
            || ()
        });
    }

    {
        let choreo_image = choreo_image.clone();
        let description = description.clone();
        let selected_dancers = selected_dancers.clone();
        use_effect_with(choreo_video_thumbnail.clone(), move |choreo_video_thumbnail| {
            save_choreography_info(
                number,
                &ChoreographyInfo {
                    choreo_image: (*choreo_image).clone(),
                    choreo_video_thumbnail: (**choreo_video_thumbnail).clone(),
                    description: (*description).clone(),
                    dancers: non_empty(&selected_dancers),
                },
            );
            || ()
        });
    }

    {
        let choreo_image = choreo_image.clone();
        let choreo_video_thumbnail = choreo_video_thumbnail.clone();
        let selected_dancers = selected_dancers.clone();
        use_effect_with(description.clone(), move |description| {
            save_choreography_info(
                number,
                &ChoreographyInfo {
                    choreo_image: (*choreo_image).clone(),
                    choreo_video_thumbnail: (*choreo_video_thumbnail).clone(),
                    description: (**description).clone(),
                    dancers: non_empty(&selected_dancers),
                },
            );
            || ()
        });
    }

    {
        let choreo_image = choreo_image.clone();
        let choreo_video_thumbnail = choreo_video_thumbnail.clone();
        let description = description.clone();
        use_effect_with(selected_dancers.clone(), move |selected_dancers| {
            save_choreography_info(
                number,
                &ChoreographyInfo {
                    choreo_image: (*choreo_image).clone(),
                    choreo_video_thumbnail: (*choreo_video_thumbnail).clone(),
                    description: (*description).clone(),
                    dancers: non_empty(selected_dancers),
                },
            );
            || ()
        });
    }

    let on_image_dropzone_click = {
        let image_input_ref = image_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = image_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    // Reads the dropped/selected image into a base64 data URL, same convention
    // used by DancerCard and the choreography video thumbnails.
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
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    if let Some(file) = file_list.get(0) {
                        on_image_file.emit(file);
                    }
                }
            }
        })
    };

    let on_image_dropzone_dragover = {
        let is_dragging_over_image = is_dragging_over_image.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            if !*is_dragging_over_image {
                is_dragging_over_image.set(true);
            }
        })
    };

    let on_image_dropzone_dragleave = {
        let is_dragging_over_image = is_dragging_over_image.clone();
        Callback::from(move |_: DragEvent| {
            is_dragging_over_image.set(false);
        })
    };

    let on_image_dropzone_drop = {
        let on_image_file = on_image_file.clone();
        let is_dragging_over_image = is_dragging_over_image.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging_over_image.set(false);
            if let Some(file) = e
                .data_transfer()
                .and_then(|dt| dt.files())
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
                Callback::from(move |data_url: String| choreo_video_thumbnail.set(Some(data_url))),
            );
        })
    };

    let on_video_file_change = {
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

    let on_video_dropzone_dragover = {
        let is_dragging_over_video = is_dragging_over_video.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            if !*is_dragging_over_video {
                is_dragging_over_video.set(true);
            }
        })
    };

    let on_video_dropzone_dragleave = {
        let is_dragging_over_video = is_dragging_over_video.clone();
        Callback::from(move |_: DragEvent| {
            is_dragging_over_video.set(false);
        })
    };

    let on_video_dropzone_drop = {
        let on_video_file = on_video_file.clone();
        let is_dragging_over_video = is_dragging_over_video.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging_over_video.set(false);
            if let Some(file) = e
                .data_transfer()
                .and_then(|dt| dt.files())
                .and_then(|files| files.get(0))
            {
                on_video_file.emit(file);
            }
        })
    };

    let on_description_input = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(textarea) = e.target_dyn_into::<HtmlTextAreaElement>() {
                description.set(textarea.value());
            }
        })
    };

    let on_dancer_select = {
        let selected_dancers = selected_dancers.clone();
        Callback::from(move |(index, name): (usize, String)| {
            let mut updated = (*selected_dancers).clone();
            if let Some(slot) = updated.get_mut(index) {
                *slot = name;
            }
            selected_dancers.set(with_trailing_empty_slot(updated));
        })
    };

    let on_dancer_remove = {
        let selected_dancers = selected_dancers.clone();
        Callback::from(move |index: usize| {
            let mut updated = (*selected_dancers).clone();
            if index < updated.len() {
                updated.remove(index);
            }
            selected_dancers.set(with_trailing_empty_slot(updated));
        })
    };

    html! {
        <div class="page about-choreo-container">
            <div class="arcadefont">
                <h2>{ format!("Info side til koreografi NR. {}", number) }</h2>

                <input
                    type="text"
                    class="choreo-title-display"
                    placeholder="Title copied to here"
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
                            <p class="info-message">{ "Drop billede" }</p>
                        } else {
                            if let Some(image) = &*choreo_image {
                                <img src={image.clone()} alt="Choreography" />
                            } else {
                                <span>{ "Upload billede" }</span>
                            }
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
                        placeholder="Beskriv forestillingen:"
                        value={(*description).clone()}
                        oninput={on_description_input}
                    />
                </div>

                <h2>{ "Choreo Video" }</h2>
                <div
                    class="dropzone"
                    onclick={on_video_dropzone_click}
                    ondragover={on_video_dropzone_dragover}
                    ondragleave={on_video_dropzone_dragleave}
                    ondrop={on_video_dropzone_drop}
                >
                    if *is_dragging_over_video {
                        <p class="info-message">{ "Drop video" }</p>
                    } else {
                        if let Some(thumbnail) = &*choreo_video_thumbnail {
                            <img src={thumbnail.clone()} alt="Choreo video thumbnail" class="video-thumbnail" />
                        } else {
                            <span>{ "Upload Choreo Video" }</span>
                        }
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
                <div class="choreo-dancers-section">
                    { for selected_dancers.iter().cloned().enumerate().map(|(index, selected_name)| {
                        let on_dancer_select = on_dancer_select.clone();
                        let on_change = Callback::from(move |e: Event| {
                            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                                on_dancer_select.emit((index, select.value()));
                            }
                        });

                        let on_remove_click = {
                            let on_dancer_remove = on_dancer_remove.clone();
                            Callback::from(move |_| on_dancer_remove.emit(index))
                        };

                        // A dancer picked in another slot is hidden here so the same
                        // dancer can't be chosen twice for this choreography.
                        let taken_elsewhere: std::collections::HashSet<&String> = selected_dancers
                            .iter()
                            .enumerate()
                            .filter(|(other_index, name)| *other_index != index && !name.is_empty())
                            .map(|(_, name)| name)
                            .collect();

                        html! {
                            <div key={index} class="choreo-dancer-row">
                                <select class="choreo-dancer-select" onchange={on_change}>
                                    <option key="" value="" selected={selected_name.is_empty()}>
                                        { "Select dancer from dropdown" }
                                    </option>
                                    { for all_dancers.iter().filter(|dancer| !taken_elsewhere.contains(&dancer.name)).map(|dancer| html! {
                                        <option key={dancer.name.clone()} value={dancer.name.clone()} selected={dancer.name == selected_name}>
                                            { dancer.name.clone() }
                                        </option>
                                    }) }
                                </select>
                                if !selected_name.is_empty() {
                                    <button type="button" class="choreo-dancer-remove" onclick={on_remove_click}>
                                        { "Fjern" }
                                    </button>
                                }
                            </div>
                        }
                    }) }
                </div>
            </div>
        </div>
    }
}
