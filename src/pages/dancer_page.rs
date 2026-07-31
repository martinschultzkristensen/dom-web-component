#![allow(dead_code)]
use crate::services::supabase::{
    create_signed_url, delete_dancer, fetch_dancers, get_current_user_id, insert_dancer,
    update_dancer, upload_dancer_image, DancerRow, NewDancer,
};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{DragEvent, Event, File, FileReader, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct DancerView {
    id: String,
    name: String,
    image_path: Option<String>,
    image_url: Option<String>,
    strength: u8,
    flexibility: u8,
}

async fn dancer_row_to_view(row: DancerRow) -> DancerView {
    let image_path = row.image_path.clone();

    let image_url = match &image_path {
        Some(path) if !path.trim().is_empty() => create_signed_url(path).await.ok(),
        _ => None,
    };

    DancerView {
        id: row.id,
        name: row.name,
        image_path,
        image_url,
        strength: row.strength,
        flexibility: row.flexibility,
    }
}

async fn dancer_rows_to_views(rows: Vec<DancerRow>) -> Vec<DancerView> {
    let mut dancers = Vec::new();

    for row in rows {
        dancers.push(dancer_row_to_view(row).await);
    }

    dancers
}

fn read_image_preview(file: File, callback: Callback<String>) {
    let Ok(reader) = FileReader::new() else {
        return;
    };

    let reader_clone = reader.clone();

    let onload = Closure::wrap(Box::new(move |_event: Event| {
        if let Ok(result) = reader_clone.result() {
            if let Some(data_url) = result.as_string() {
                callback.emit(data_url);
            }
        }
    }) as Box<dyn FnMut(Event)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    let _ = reader.read_as_data_url(&file);
}

#[function_component(DancerPage)]
pub fn dancer_page() -> Html {
    let dancers = use_state(Vec::<DancerView>::new);
    let loading_dancers = use_state(|| true);
    let page_error = use_state(|| None::<String>);
    let reload_counter = use_state(|| 0usize);

    let is_add_open = use_state(|| false);
    let is_saving = use_state(|| false);
    let is_deleting = use_state(|| false);

    let add_name = use_state(String::new);
    let add_strength = use_state(|| 5u8);
    let add_flexibility = use_state(|| 5u8);
    let add_image_preview = use_state(|| None::<String>);
    let add_image_file = use_state(|| None::<File>);
    let add_error = use_state(|| None::<String>);
    let add_image_input_ref = use_node_ref();

    let editing_dancer_id = use_state(|| None::<String>);
    let edit_name = use_state(String::new);
    let edit_strength = use_state(|| 5u8);
    let edit_flexibility = use_state(|| 5u8);
    let edit_image_preview = use_state(|| None::<String>);
    let edit_image_path = use_state(|| None::<String>);
    let edit_image_file = use_state(|| None::<File>);
    let edit_error = use_state(|| None::<String>);
    let edit_image_input_ref = use_node_ref();

    let is_dragging_add_image = use_state(|| false);
    let is_dragging_edit_image = use_state(|| false);

    {
        let dancers = dancers.clone();
        let loading_dancers = loading_dancers.clone();
        let page_error = page_error.clone();

        use_effect_with(*reload_counter, move |_| {
            spawn_local(async move {
                loading_dancers.set(true);

                match fetch_dancers().await {
                    Ok(rows) => {
                        let loaded_dancers = dancer_rows_to_views(rows).await;
                        dancers.set(loaded_dancers);
                        page_error.set(None);
                    }
                    Err(message) => {
                        dancers.set(Vec::new());
                        page_error.set(Some(message));
                    }
                }

                loading_dancers.set(false);
            });

            || ()
        });
    }

    let on_open_add = {
        let is_add_open = is_add_open.clone();

        Callback::from(move |_| {
            is_add_open.set(true);
        })
    };

    let on_cancel_add = {
        let is_add_open = is_add_open.clone();
        let add_name = add_name.clone();
        let add_strength = add_strength.clone();
        let add_flexibility = add_flexibility.clone();
        let add_image_preview = add_image_preview.clone();
        let add_image_file = add_image_file.clone();
        let add_error = add_error.clone();

        Callback::from(move |_| {
            is_add_open.set(false);
            add_name.set(String::new());
            add_strength.set(5);
            add_flexibility.set(5);
            add_image_preview.set(None);
            add_image_file.set(None);
            add_error.set(None);
        })
    };

    let on_add_name_input = {
        let add_name = add_name.clone();
        let add_error = add_error.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                add_name.set(input.value());
                add_error.set(None);
            }
        })
    };

    let on_add_strength_input = {
        let add_strength = add_strength.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<u8>() {
                    add_strength.set(value);
                }
            }
        })
    };

    let on_add_flexibility_input = {
        let add_flexibility = add_flexibility.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<u8>() {
                    add_flexibility.set(value);
                }
            }
        })
    };

    let on_add_image_file = {
        let add_image_preview = add_image_preview.clone();
        let add_image_file = add_image_file.clone();
        let add_error = add_error.clone();

        Callback::from(move |file: File| {
            add_image_file.set(Some(file.clone()));
            add_error.set(None);

            let add_image_preview = add_image_preview.clone();

            read_image_preview(
                file,
                Callback::from(move |data_url: String| {
                    add_image_preview.set(Some(data_url));
                }),
            );
        })
    };

    let on_add_image_click = {
        let add_image_input_ref = add_image_input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = add_image_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_add_image_change = {
        let on_add_image_file = on_add_image_file.clone();

        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        on_add_image_file.emit(file);
                    }
                }
            }
        })
    };

    let on_add_image_dragover = {
        let is_dragging_add_image = is_dragging_add_image.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_add_image.set(true);
        })
    };

    let on_add_image_dragleave = {
        let is_dragging_add_image = is_dragging_add_image.clone();

        Callback::from(move |_event: DragEvent| {
            is_dragging_add_image.set(false);
        })
    };

    let on_add_image_drop = {
        let is_dragging_add_image = is_dragging_add_image.clone();
        let on_add_image_file = on_add_image_file.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_add_image.set(false);

            if let Some(file) = event
                .data_transfer()
                .and_then(|data_transfer| data_transfer.files())
                .and_then(|files| files.get(0))
            {
                on_add_image_file.emit(file);
            }
        })
    };

    let on_save_add = {
        let dancers = dancers.clone();
        let is_saving = is_saving.clone();
        let add_name = add_name.clone();
        let add_strength = add_strength.clone();
        let add_flexibility = add_flexibility.clone();
        let add_image_preview = add_image_preview.clone();
        let add_image_file = add_image_file.clone();
        let add_error = add_error.clone();
        let is_add_open = is_add_open.clone();
        let reload_counter = reload_counter.clone();
        let page_error = page_error.clone();

        Callback::from(move |_| {
            if *is_saving {
                return;
            }

            let name_value = (*add_name).trim().to_string();
            let strength_value = *add_strength;
            let flexibility_value = *add_flexibility;
            let selected_image_file = (*add_image_file).clone();

            if name_value.is_empty() {
                add_error.set(Some("Please enter dancer name.".to_string()));
                return;
            }

            let Some(selected_image_file) = selected_image_file else {
                add_error.set(Some("Please choose a dancer image.".to_string()));
                return;
            };

            let Some(user_id) = get_current_user_id() else {
                add_error.set(Some("User is not logged in.".to_string()));
                return;
            };

            is_saving.set(true);
            add_error.set(None);
            page_error.set(None);

            let dancers = dancers.clone();
            let is_saving = is_saving.clone();
            let add_name = add_name.clone();
            let add_strength = add_strength.clone();
            let add_flexibility = add_flexibility.clone();
            let add_image_preview = add_image_preview.clone();
            let add_image_file = add_image_file.clone();
            let add_error = add_error.clone();
            let is_add_open = is_add_open.clone();
            let reload_counter = reload_counter.clone();

            spawn_local(async move {
                let image_path = match upload_dancer_image(selected_image_file).await {
                    Ok(path) => Some(path),
                    Err(message) => {
                        add_error.set(Some(message));
                        is_saving.set(false);
                        return;
                    }
                };

                let new_dancer = NewDancer {
                    created_by: user_id,
                    name: name_value,
                    image_path,
                    strength: strength_value,
                    flexibility: flexibility_value,
                };

                match insert_dancer(new_dancer).await {
                    Ok(_) => {
                        add_name.set(String::new());
                        add_strength.set(5);
                        add_flexibility.set(5);
                        add_image_preview.set(None);
                        add_image_file.set(None);
                        is_add_open.set(false);
                        dancers.set(Vec::new());
                        reload_counter.set(*reload_counter + 1);
                    }
                    Err(message) => {
                        add_error.set(Some(message));
                    }
                }

                is_saving.set(false);
            });
        })
    };

    let on_cancel_edit = {
        let editing_dancer_id = editing_dancer_id.clone();
        let edit_error = edit_error.clone();
        let edit_image_file = edit_image_file.clone();

        Callback::from(move |_| {
            editing_dancer_id.set(None);
            edit_error.set(None);
            edit_image_file.set(None);
        })
    };

    let on_edit_name_input = {
        let edit_name = edit_name.clone();
        let edit_error = edit_error.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                edit_name.set(input.value());
                edit_error.set(None);
            }
        })
    };

    let on_edit_strength_input = {
        let edit_strength = edit_strength.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<u8>() {
                    edit_strength.set(value);
                }
            }
        })
    };

    let on_edit_flexibility_input = {
        let edit_flexibility = edit_flexibility.clone();

        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<u8>() {
                    edit_flexibility.set(value);
                }
            }
        })
    };

    let on_edit_image_file = {
        let edit_image_preview = edit_image_preview.clone();
        let edit_image_file = edit_image_file.clone();
        let edit_error = edit_error.clone();

        Callback::from(move |file: File| {
            edit_image_file.set(Some(file.clone()));
            edit_error.set(None);

            let edit_image_preview = edit_image_preview.clone();

            read_image_preview(
                file,
                Callback::from(move |data_url: String| {
                    edit_image_preview.set(Some(data_url));
                }),
            );
        })
    };

    let on_edit_image_click = {
        let edit_image_input_ref = edit_image_input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = edit_image_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_edit_image_change = {
        let on_edit_image_file = on_edit_image_file.clone();

        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        on_edit_image_file.emit(file);
                    }
                }
            }
        })
    };

    let on_edit_image_dragover = {
        let is_dragging_edit_image = is_dragging_edit_image.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_edit_image.set(true);
        })
    };

    let on_edit_image_dragleave = {
        let is_dragging_edit_image = is_dragging_edit_image.clone();

        Callback::from(move |_event: DragEvent| {
            is_dragging_edit_image.set(false);
        })
    };

    let on_edit_image_drop = {
        let is_dragging_edit_image = is_dragging_edit_image.clone();
        let on_edit_image_file = on_edit_image_file.clone();

        Callback::from(move |event: DragEvent| {
            event.prevent_default();
            is_dragging_edit_image.set(false);

            if let Some(file) = event
                .data_transfer()
                .and_then(|data_transfer| data_transfer.files())
                .and_then(|files| files.get(0))
            {
                on_edit_image_file.emit(file);
            }
        })
    };

    let selected_edit_id = (*editing_dancer_id).clone();

    html! {
        <div class="page about-choreo-container">
            <h2>{ "Dancer Page" }</h2>

            <div class="creator-help-box">
                <p>
                    { "Create dancers that can be used when building a choreography." }
                </p>
                
            </div>

            if let Some(message) = &*page_error {
                <p class="error-message">{ message }</p>
            }

            <div class="main-panel">
                <button
                    type="button"
                    class="main-action-button"
                    onclick={on_open_add}
                >
                    { "+ Add Dancer" }
                </button>
            </div>

            if *is_add_open {
                <div class="dancer-editor-panel">
                    <h2>{ "Add Dancer" }</h2>

                    <div class="dancer-editor-grid">
                        <div
                            class="dancer-image-dropzone"
                            onclick={on_add_image_click}
                            ondragover={on_add_image_dragover}
                            ondragleave={on_add_image_dragleave}
                            ondrop={on_add_image_drop}
                        >
                            if *is_dragging_add_image {
                                <p class="info-message">{ "Drop image" }</p>
                            } else if let Some(preview) = &*add_image_preview {
                                <img src={preview.clone()} alt="Dancer preview" />
                            } else {
                                <span>{ "Upload Dancer Image" }</span>
                            }

                            <input
                                type="file"
                                accept="image/*"
                                ref={add_image_input_ref}
                                style="display: none;"
                                onchange={on_add_image_change}
                            />
                        </div>

                        <div class="dancer-editor-fields">
                            <input
                                type="text"
                                class="dancer-text-input"
                                placeholder="Dancer name"
                                value={(*add_name).clone()}
                                oninput={on_add_name_input}
                            />

                            <label>{ format!("Strength: {}", *add_strength) }</label>
                            <input
                                type="range"
                                min="0"
                                max="10"
                                value={add_strength.to_string()}
                                oninput={on_add_strength_input}
                            />

                            <label>{ format!("Flexibility: {}", *add_flexibility) }</label>
                            <input
                                type="range"
                                min="0"
                                max="10"
                                value={add_flexibility.to_string()}
                                oninput={on_add_flexibility_input}
                            />

                            if let Some(message) = &*add_error {
                                <p class="error-message">{ message }</p>
                            }

                            <div class="dancer-editor-actions">
                                <button
                                    type="button"
                                    class="main-action-button"
                                    onclick={on_save_add}
                                    disabled={*is_saving}
                                >
                                    {
                                        if *is_saving {
                                            "Saving..."
                                        } else {
                                            "Save Dancer"
                                        }
                                    }
                                </button>

                                <button
                                    type="button"
                                    class="small-remove-button"
                                    onclick={on_cancel_add}
                                >
                                    { "Cancel" }
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            }

            <h2>{ "My Dancers" }</h2>

            if *loading_dancers {
                <p class="login-help-text">{ "Loading dancers..." }</p>
            }

            if !*loading_dancers && dancers.is_empty() {
                <p class="login-help-text">
                    { "No dancers yet. Click + Add Dancer to create the first dancer." }
                </p>
            }

            if !dancers.is_empty() {
                <div class="dancer-card-grid">
                    {
                        for dancers.iter().map(|dancer| {
                            let is_editing_this = selected_edit_id.as_ref() == Some(&dancer.id);

                            let on_start_edit = {
                                let editing_dancer_id = editing_dancer_id.clone();
                                let edit_name = edit_name.clone();
                                let edit_strength = edit_strength.clone();
                                let edit_flexibility = edit_flexibility.clone();
                                let edit_image_preview = edit_image_preview.clone();
                                let edit_image_path = edit_image_path.clone();
                                let edit_image_file = edit_image_file.clone();
                                let edit_error = edit_error.clone();

                                let dancer = dancer.clone();

                                Callback::from(move |_| {
                                    editing_dancer_id.set(Some(dancer.id.clone()));
                                    edit_name.set(dancer.name.clone());
                                    edit_strength.set(dancer.strength);
                                    edit_flexibility.set(dancer.flexibility);
                                    edit_image_preview.set(dancer.image_url.clone());
                                    edit_image_path.set(dancer.image_path.clone());
                                    edit_image_file.set(None);
                                    edit_error.set(None);
                                })
                            };

                            let on_save_edit = {
                                let is_saving = is_saving.clone();
                                let edit_name = edit_name.clone();
                                let edit_strength = edit_strength.clone();
                                let edit_flexibility = edit_flexibility.clone();
                                let edit_image_path = edit_image_path.clone();
                                let edit_image_file = edit_image_file.clone();
                                let edit_error = edit_error.clone();
                                let editing_dancer_id = editing_dancer_id.clone();
                                let reload_counter = reload_counter.clone();
                                let page_error = page_error.clone();

                                let dancer_id = dancer.id.clone();

                                Callback::from(move |_| {
                                    if *is_saving {
                                        return;
                                    }

                                    let name_value = (*edit_name).trim().to_string();

                                    if name_value.is_empty() {
                                        edit_error.set(Some("Please enter dancer name.".to_string()));
                                        return;
                                    }

                                    is_saving.set(true);
                                    edit_error.set(None);
                                    page_error.set(None);

                                    let is_saving = is_saving.clone();
                                    let edit_strength = edit_strength.clone();
                                    let edit_flexibility = edit_flexibility.clone();
                                    let edit_image_path = edit_image_path.clone();
                                    let edit_image_file = edit_image_file.clone();
                                    let edit_error = edit_error.clone();
                                    let editing_dancer_id = editing_dancer_id.clone();
                                    let reload_counter = reload_counter.clone();
                                    let dancer_id = dancer_id.clone();

                                    spawn_local(async move {
                                        let image_path = match (*edit_image_file).clone() {
                                            Some(file) => match upload_dancer_image(file).await {
                                                Ok(path) => Some(path),
                                                Err(message) => {
                                                    edit_error.set(Some(message));
                                                    is_saving.set(false);
                                                    return;
                                                }
                                            },
                                            None => (*edit_image_path).clone(),
                                        };

                                        match update_dancer(
                                            dancer_id,
                                            name_value,
                                            image_path,
                                            *edit_strength,
                                            *edit_flexibility,
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                editing_dancer_id.set(None);
                                                reload_counter.set(*reload_counter + 1);
                                            }
                                            Err(message) => {
                                                edit_error.set(Some(message));
                                            }
                                        }

                                        is_saving.set(false);
                                    });
                                })
                            };

                            let on_remove_dancer = {
                                let is_deleting = is_deleting.clone();
                                let reload_counter = reload_counter.clone();
                                let page_error = page_error.clone();
                                let dancer_id = dancer.id.clone();
                                let dancer_name = dancer.name.clone();

                                Callback::from(move |_| {
                                    if *is_deleting {
                                        return;
                                    }

                                    let should_remove = web_sys::window()
                                        .and_then(|window| {
                                            window
                                                .confirm_with_message(&format!("Remove dancer {}?", dancer_name))
                                                .ok()
                                        })
                                        .unwrap_or(false);

                                    if !should_remove {
                                        return;
                                    }

                                    is_deleting.set(true);
                                    page_error.set(None);

                                    let is_deleting = is_deleting.clone();
                                    let reload_counter = reload_counter.clone();
                                    let page_error = page_error.clone();
                                    let dancer_id = dancer_id.clone();

                                    spawn_local(async move {
                                        match delete_dancer(dancer_id).await {
                                            Ok(_) => {
                                                reload_counter.set(*reload_counter + 1);
                                            }
                                            Err(message) => {
                                                page_error.set(Some(message));
                                            }
                                        }

                                        is_deleting.set(false);
                                    });
                                })
                            };

                            html! {
                                <div
                                        key={dancer.id.clone()}
                                        class={classes!(
                                            "dancer-card-new",
                                            if is_editing_this {
                                                Some("dancer-card-editing")
                                            } else {
                                                None
                                            }
                                        )}
                                    >
                                    if is_editing_this {
                                        <div class="dancer-editor-grid">
                                            <div
                                                class="dancer-image-dropzone"
                                                onclick={on_edit_image_click.clone()}
                                                ondragover={on_edit_image_dragover.clone()}
                                                ondragleave={on_edit_image_dragleave.clone()}
                                                ondrop={on_edit_image_drop.clone()}
                                            >
                                                if *is_dragging_edit_image {
                                                    <p class="info-message">{ "Drop image" }</p>
                                                } else if let Some(preview) = &*edit_image_preview {
                                                    <img src={preview.clone()} alt="Dancer preview" />
                                                } else {
                                                    <span>{ "Upload Dancer Image" }</span>
                                                }

                                                <input
                                                    type="file"
                                                    accept="image/*"
                                                    ref={edit_image_input_ref.clone()}
                                                    style="display: none;"
                                                    onchange={on_edit_image_change.clone()}
                                                />
                                            </div>

                                            <div class="dancer-editor-fields">
                                                <input
                                                    type="text"
                                                    class="dancer-text-input"
                                                    placeholder="Dancer name"
                                                    value={(*edit_name).clone()}
                                                    oninput={on_edit_name_input.clone()}
                                                />

                                                <label>{ format!("Strength: {}", *edit_strength) }</label>
                                                <input
                                                    type="range"
                                                    min="0"
                                                    max="10"
                                                    value={edit_strength.to_string()}
                                                    oninput={on_edit_strength_input.clone()}
                                                />

                                                <label>{ format!("Flexibility: {}", *edit_flexibility) }</label>
                                                <input
                                                    type="range"
                                                    min="0"
                                                    max="10"
                                                    value={edit_flexibility.to_string()}
                                                    oninput={on_edit_flexibility_input.clone()}
                                                />

                                                if let Some(message) = &*edit_error {
                                                    <p class="error-message">{ message }</p>
                                                }

                                                <div class="dancer-editor-actions">
                                                    <button
                                                        type="button"
                                                        class="main-action-button"
                                                        onclick={on_save_edit}
                                                        disabled={*is_saving}
                                                    >
                                                        {
                                                            if *is_saving {
                                                                "Saving..."
                                                            } else {
                                                                "Save Changes"
                                                            }
                                                        }
                                                    </button>

                                                    <button
                                                        type="button"
                                                        class="small-remove-button"
                                                        onclick={on_cancel_edit.clone()}
                                                    >
                                                        { "Cancel" }
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    } else {
                                        if let Some(image_url) = &dancer.image_url {
                                            <img
                                                src={image_url.clone()}
                                                alt={format!("Image of {}", dancer.name)}
                                                class="dancer-card-image"
                                            />
                                        } else {
                                            <div class="dancer-card-image-placeholder">
                                                { "Image unavailable" }
                                            </div>
                                        }

                                        <div class="dancer-card-content">
                                            <h3>{ dancer.name.clone() }</h3>

                                            <div class="dancer-stat-line">
                                                <span>{ format!("Strength: {}", dancer.strength) }</span>
                                                <div class="dancer-stat-bar">
                                                    <div
                                                        class="dancer-stat-fill"
                                                        style={format!("width: {}%", dancer.strength * 10)}
                                                    />
                                                </div>
                                            </div>

                                            <div class="dancer-stat-line">
                                                <span>{ format!("Flexibility: {}", dancer.flexibility) }</span>
                                                <div class="dancer-stat-bar">
                                                    <div
                                                        class="dancer-stat-fill"
                                                        style={format!("width: {}%", dancer.flexibility * 10)}
                                                    />
                                                </div>
                                            </div>

                                            <div class="dancer-card-actions">
                                                <button
                                                    type="button"
                                                    class="dancer-secondary-button"
                                                    onclick={on_start_edit}
                                                >
                                                    { "Edit" }
                                                </button>

                                                <button
                                                    type="button"
                                                    class="small-remove-button"
                                                    onclick={on_remove_dancer}
                                                >
                                                    { "Remove" }
                                                </button>
                                            </div>
                                        </div>
                                    }
                                </div>
                            }
                        })
                    }
                </div>
            }
        </div>
    }
}