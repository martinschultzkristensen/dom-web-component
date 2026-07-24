//src/pages/sub_page/info_choreo_page.rs
use crate::components::molecules::video_list::ChoreographyEntry;
use crate::pages::choreography_page::DRAFT_CHOREOGRAPHIES_STORAGE_KEY;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{DragEvent, Event, File, FileReader, HtmlInputElement, HtmlTextAreaElement};
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
    description: String,
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

    // Choreo image + description are persisted to localStorage (per choreography number)
    // so edits survive a refresh, same convention as choreography_page.rs.
    let choreo_image = use_state(|| load_choreography_info(number).choreo_image);
    let description = use_state(|| load_choreography_info(number).description);
    let is_dragging_over = use_state(|| false);
    let file_input_ref = use_node_ref();

    {
        let description = description.clone();
        use_effect_with(choreo_image.clone(), move |choreo_image| {
            save_choreography_info(
                number,
                &ChoreographyInfo {
                    choreo_image: (**choreo_image).clone(),
                    description: (*description).clone(),
                },
            );
            || ()
        });
    }

    {
        let choreo_image = choreo_image.clone();
        use_effect_with(description.clone(), move |description| {
            save_choreography_info(
                number,
                &ChoreographyInfo {
                    choreo_image: (*choreo_image).clone(),
                    description: (**description).clone(),
                },
            );
            || ()
        });
    }

    let on_dropzone_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
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

    let on_file_change = {
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
        let on_image_file = on_image_file.clone();
        let is_dragging_over = is_dragging_over.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            is_dragging_over.set(false);
            if let Some(file) = e
                .data_transfer()
                .and_then(|dt| dt.files())
                .and_then(|files| files.get(0))
            {
                on_image_file.emit(file);
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
                        onclick={on_dropzone_click}
                        ondragover={on_dropzone_dragover}
                        ondragleave={on_dropzone_dragleave}
                        ondrop={on_dropzone_drop}
                    >
                        if *is_dragging_over {
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
                            ref={file_input_ref}
                            style="display: none;"
                            onchange={on_file_change}
                        />
                    </div>

                    <textarea
                        class="choreo-description-input"
                        placeholder="Beskriv forestillingen:"
                        value={(*description).clone()}
                        oninput={on_description_input}
                    />
                </div>
            </div>
        </div>
    }
}
