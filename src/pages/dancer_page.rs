//src/pages/dancer_page.rs
use crate::components::atoms::dancer::{DancerCard, DancerData};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{Event, FileReader, HtmlInputElement};
use yew::prelude::*;

#[function_component(DancerPage)]
pub fn dancer_page() -> Html {
    // List of dancers added so far
    let dancers = use_state(Vec::<DancerData>::new);

    // Form field state
    let name = use_state(String::new);
    let name_error = use_state(String::new);
    let strength = use_state(|| 5u8);
    let flexibility = use_state(|| 5u8);
    let image_data = use_state(|| Option::<String>::None);

    // --- Handlers ---
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

    // Reads the selected file into a base64 data URL so it can be used
    // directly as an <img src=...> without needing backend involvement.
    let on_image_change = {
        let image_data = image_data.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    if let Some(file) = file_list.get(0) {
                        let reader = FileReader::new().unwrap();
                        let reader_clone = reader.clone();
                        let image_data = image_data.clone();

                        let onload = Closure::wrap(Box::new(move |_event: Event| {
                            if let Ok(result) = reader_clone.result() {
                                if let Some(data_url) = result.as_string() {
                                    image_data.set(Some(data_url));
                                }
                            }
                        })
                            as Box<dyn FnMut(Event)>);

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

        Callback::from(move |_| {
            // Require at least a name before adding
            if (*name).trim().is_empty() {
                name_error.set(format!("enter {}", dancer_name_placeholder));
                return;
            }

            name_error.set(String::new());

            let new_dancer = DancerData {
                image: (*image_data).clone().unwrap_or_default(),
                name: (*name).clone(),
                strength: *strength,
                flexibility: *flexibility,
            };

            let mut updated = (*dancers).clone();
            updated.push(new_dancer);
            dancers.set(updated);

            // Reset form fields for the next entry
            name.set(String::new());
            strength.set(5);
            flexibility.set(5);
            image_data.set(None);
        })
    };

    html! {
        <div class="page about-choreo-container">
            <div class="arcadefont">
                <h2>{ "Dancer Page" }</h2>

                // --- Add Dancer Form ---
                <div class="info-section-container">
                    <div class="description">
                        <p>{ "Add a new dancer" }</p>

                        <input
                            type="file"
                            accept="image/*"
                            onchange={on_image_change}
                        />

                        <br/>

                        <input
                            type="text"
                            placeholder={dancer_name_placeholder}
                            value={(*name).clone()}
                            oninput={on_name_input}
                        />

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
                    <div class="add-dancer-panel">
                        <button class="main-action-button" onclick={on_add_dancer}>
                            { "Add Dancer" }
                        </button>

                        
                    </div>
                </div>
                </div>

                <h2>{ "Dancers" }</h2>
                {
                    (*dancers).iter().map(|dancer| {
                        html! {
                            <DancerCard dancer={dancer.clone()} />
                        }
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}
