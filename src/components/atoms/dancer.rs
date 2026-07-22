//components/atoms/dancer.rs
//Purpose of code: Create a dancer struct which can be used in src/components/data/choreography_data.rs
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, FileReader, HtmlInputElement};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}
#[derive(Clone, PartialEq, Properties, Serialize, Deserialize)]
pub struct DancerData {
    pub image: String,
    pub name: String,
    pub strength: u8,
    pub flexibility: u8,
}

impl DancerData {
    fn new(image: String, name: String, strength: u8, flexibility: u8) -> Self {
        Self {
            image,
            name,
            strength,
            flexibility,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct StatBarProps {
    pub value: u8,
    pub label: String,
}

#[function_component(StatBar)]
fn stat_bar(props: &StatBarProps) -> Html {
    let percentage: u8 = props.value * 10;

    html! {
        <div class="stat-container" style={format!("--stat-percentage: {}%", percentage)}>
            <span class="stat-label">{&props.label}</span>
            <div class="stat-bar-border">
                <div class="stat-bar-fill" style={format!("width: {}%", percentage)}></div>
            </div>
            // <span class="stat-value">{props.value}</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DancerCardProps {
    pub dancer: DancerData,
    #[prop_or_default]
    pub on_image_update: Callback<String>,
}

#[function_component(DancerCard)]
pub fn dancer_card(props: &DancerCardProps) -> Html {
    let dancer = &props.dancer;

    // Image path resolution state
    let img_src = {
        let initial = dancer.image.clone();
        use_state(|| initial)
    };

    {
        let img_src = img_src.clone();
        let img_path = dancer.image.clone();
        use_effect_with(img_path.clone(), move |img_path| {
            if img_path.starts_with("media/") {
                wasm_bindgen_futures::spawn_local({
                    let img_src = img_src.clone();
                    let img_path = img_path.clone();
                    async move {
                        let js_args =
                            serde_wasm_bindgen::to_value(&json!({ "path": img_path })).unwrap();
                        let result = invoke("resolve_media_path", js_args).await;
                        match serde_wasm_bindgen::from_value::<String>(result) {
                            Ok(resolved) => img_src.set(resolved),
                            Err(_) => img_src.set(img_path),
                        }
                    }
                });
            } else {
                img_src.set(img_path.clone());
            }
            || ()
        });
    }

    let file_input_ref = use_node_ref();

    let on_add_image_click = {
        let file_input_ref = file_input_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_file_change = {
        let img_src = img_src.clone();
        let on_image_update = props.on_image_update.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Some(file_list) = input.files() {
                    if let Some(file) = file_list.get(0) {
                        let reader = FileReader::new().unwrap();
                        let reader_clone = reader.clone();
                        let img_src = img_src.clone();
                        let on_image_update = on_image_update.clone();

                        let onload = Closure::wrap(Box::new(move |_event: Event| {
                            if let Ok(result) = reader_clone.result() {
                                if let Some(data_url) = result.as_string() {
                                    img_src.set(data_url.clone());
                                    on_image_update.emit(data_url);
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

    html! {
        <div class="info-section-container">
            if (*img_src).is_empty() {
                <button class="main-action-button" onclick={on_add_image_click}>
                    { "Add Image" }
                </button>
            } else {
                <img src={(*img_src).clone()} alt={format!("Image of {}", dancer.name)} />
            }
            <input
                type="file"
                accept="image/*"
                ref={file_input_ref}
                style="display: none;"
                onchange={on_file_change}
            />
            <div class="name-and-stats-container">
            <p>{&dancer.name}</p>
                <StatBar value={dancer.strength} label="strength" />
                <StatBar value={dancer.flexibility} label="flexibility" />
            </div>
        </div>
    }
}
