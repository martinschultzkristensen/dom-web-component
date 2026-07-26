use crate::components::atoms::dancer::{DancerCard, DancerData};
use crate::services::supabase::{
    create_signed_url, fetch_dancers, get_current_user_id, insert_dancer, upload_dancer_image,
    DancerRow, NewDancer,
};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, File, FileReader, HtmlInputElement};
use yew::prelude::*;

const DANCERS_STORAGE_KEY: &str = "dancers";

fn save_dancers_to_local_cache(dancers: &[DancerData]) {
    if let Ok(json) = serde_json::to_string(dancers) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(DANCERS_STORAGE_KEY, &json);
        }
    }
}

async fn dancer_rows_to_data(rows: Vec<DancerRow>) -> Vec<DancerData> {
    let mut dancers = Vec::new();

    for row in rows {
        let image = match row.image_path {
            Some(path) if !path.trim().is_empty() => {
                create_signed_url(&path).await.unwrap_or_default()
            }
            _ => String::new(),
        };

        dancers.push(DancerData {
            image,
            name: row.name,
            strength: row.strength,
            flexibility: row.flexibility,
        });
    }

    dancers
}

#[function_component(DancerPage)]
pub fn dancer_page() -> Html {
    let dancers = use_state(Vec::<DancerData>::new);
    let loading_dancers = use_state(|| true);
    let page_error = use_state(|| None::<String>);
    let is_saving = use_state(|| false);

    // Load dancers from Supabase when page opens.
    {
        let dancers = dancers.clone();
        let loading_dancers = loading_dancers.clone();
        let page_error = page_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                loading_dancers.set(true);

                match fetch_dancers().await {
                    Ok(rows) => {
                        let loaded_dancers = dancer_rows_to_data(rows).await;
                        save_dancers_to_local_cache(&loaded_dancers);
                        dancers.set(loaded_dancers);
                        page_error.set(None);
                    }
                    Err(message) => {
                        page_error.set(Some(message));
                    }
                }

                loading_dancers.set(false);
            });

            || ()
        });
    }

    // Form field state
    let name = use_state(String::new);
    let name_error = use_state(String::new);
    let strength = use_state(|| 5u8);
    let flexibility = use_state(|| 5u8);
    let image_data = use_state(|| Option::<String>::None);
    let image_file = use_state(|| Option::<File>::None);

    let on_name_input = {
        let name = name.clone();
        let name_error = name_error.clone();

        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                name.set(input.value());
                name_error.set(String::new());
            }
        })
    };

    let on_strength_input = {
        let strength = strength.clone();

        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(val) = input.value().parse::<u8>() {
                    strength.set(val);
                }
            }
        })
    };

    let on_flexibility_input = {
        let flexibility = flexibility.clone();

        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(val) = input.value().parse::<u8>() {
                    flexibility.set(val);
                }
            }
        })
    };

    let on_image_change = {
        let image_data = image_data.clone();
        let image_file = image_file.clone();

        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    if let Some(file) = file_list.get(0) {
                        image_file.set(Some(file.clone()));

                        let reader = FileReader::new().unwrap();
                        let reader_clone = reader.clone();
                        let image_data = image_data.clone();

                        let onload = Closure::wrap(Box::new(move |_event: Event| {
                            if let Ok(result) = reader_clone.result() {
                                if let Some(data_url) = result.as_string() {
                                    image_data.set(Some(data_url));
                                }
                            }
                        }) as Box<dyn FnMut(Event)>);

                        reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                        onload.forget();
                        reader.read_as_data_url(&file).unwrap();
                    }
                }
            }
        })
    };

    let dancer_name_placeholder = "Dancer Name";

    let on_add_dancer = {
        let dancers = dancers.clone();
        let name = name.clone();
        let name_error = name_error.clone();
        let strength = strength.clone();
        let flexibility = flexibility.clone();
        let image_data = image_data.clone();
        let image_file = image_file.clone();
        let page_error = page_error.clone();
        let is_saving = is_saving.clone();

        Callback::from(move |_| {
            let name_value = (*name).trim().to_string();
            let strength_value = *strength;
            let flexibility_value = *flexibility;
            let selected_image_file = (*image_file).clone();

            if name_value.is_empty() {
                name_error.set(format!("enter {}", dancer_name_placeholder));
                return;
            }

            if selected_image_file.is_none() {
                page_error.set(Some("Please choose an image before adding a dancer.".to_string()));
                return;
            }

            let Some(user_id) = get_current_user_id() else {
                page_error.set(Some("User is not logged in.".to_string()));
                return;
            };

            name_error.set(String::new());
            page_error.set(None);
            is_saving.set(true);

            let dancers = dancers.clone();
            let name = name.clone();
            let strength = strength.clone();
            let flexibility = flexibility.clone();
            let image_data = image_data.clone();
            let image_file = image_file.clone();
            let page_error = page_error.clone();
            let is_saving = is_saving.clone();

            spawn_local(async move {
                let image_path = match selected_image_file {
                    Some(file) => match upload_dancer_image(file).await {
                        Ok(path) => Some(path),
                        Err(message) => {
                            page_error.set(Some(message));
                            is_saving.set(false);
                            return;
                        }
                    },
                    None => None,
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
                        match fetch_dancers().await {
                            Ok(rows) => {
                                let loaded_dancers = dancer_rows_to_data(rows).await;
                                save_dancers_to_local_cache(&loaded_dancers);
                                dancers.set(loaded_dancers);
                            }
                            Err(message) => {
                                page_error.set(Some(message));
                            }
                        }

                        name.set(String::new());
                        strength.set(5);
                        flexibility.set(5);
                        image_data.set(None);
                        image_file.set(None);
                    }
                    Err(message) => {
                        page_error.set(Some(message));
                    }
                }

                is_saving.set(false);
            });
        })
    };

    html! {
        <div class="page about-choreo-container">
            <h2>{ "Dancer Page" }</h2>

            if let Some(message) = &*page_error {
                <p class="error-message">{ message }</p>
            }

            if *loading_dancers {
                <p>{ "Loading dancers..." }</p>
            }

            <div class="info-section-container">
                <div class="description">
                    <p>{ "Add a new dancer" }</p>

                    <input
                        type="text"
                        placeholder={dancer_name_placeholder}
                        value={(*name).clone()}
                        oninput={on_name_input}
                    />

                    <br/>

                    <input
                        type="file"
                        accept="image/*"
                        onchange={on_image_change}
                    />

                    <p class="login-help-text">
                        { "Choose an image. It will be uploaded to Supabase Storage." }
                    </p>

                    if let Some(preview) = &*image_data {
                        <div class="dancer-image-preview">
                            <img src={preview.clone()} alt="Selected dancer preview" />
                        </div>
                    }

                    <br/>

                    <label>{ format!("Strength: {}", *strength) }</label>
                    <input
                        type="range"
                        min="0"
                        max="10"
                        value={strength.to_string()}
                        oninput={on_strength_input}
                    />

                    <br/>

                    <label>{ format!("Flexibility: {}", *flexibility) }</label>
                    <input
                        type="range"
                        min="0"
                        max="10"
                        value={flexibility.to_string()}
                        oninput={on_flexibility_input}
                    />

                    <br/>

                    if !(*name_error).is_empty() {
                        <p class="error-message">{ (*name_error).clone() }</p>
                    }

                    <div class="main-panel">
                        <button
                            class="main-action-button"
                            onclick={on_add_dancer}
                            disabled={*is_saving}
                        >
                            {
                                if *is_saving {
                                    "Saving..."
                                } else {
                                    "Add Dancer"
                                }
                            }
                        </button>
                    </div>
                </div>
            </div>

            <h2>{ "Dancers" }</h2>

            {
                (*dancers).iter().enumerate().rev().map(|(idx, dancer)| {
                    let on_image_update = {
                        let dancers = dancers.clone();

                        Callback::from(move |data_url: String| {
                            let mut updated = (*dancers).clone();

                            if let Some(d) = updated.get_mut(idx) {
                                d.image = data_url;
                            }

                            dancers.set(updated);
                        })
                    };

                    let on_name_update = {
                        let dancers = dancers.clone();

                        Callback::from(move |new_name: String| {
                            let mut updated = (*dancers).clone();

                            if let Some(d) = updated.get_mut(idx) {
                                d.name = new_name;
                            }

                            dancers.set(updated);
                        })
                    };

                    html! {
                        <DancerCard
                            dancer={dancer.clone()}
                            on_image_update={on_image_update}
                            on_name_update={on_name_update}
                        />
                    }
                }).collect::<Html>()
            }
        </div>
    }
}