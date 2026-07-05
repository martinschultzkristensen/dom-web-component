//components/atoms/dancer.rs
//Purpose of code: Create a dancer struct which can be used in src/components/data/choreography_data.rs
use serde_json::json;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}
#[derive(Clone, PartialEq, Properties)]
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

    html! {
        <div class="info-section-container">
            <img src={(*img_src).clone()} alt={format!("Image of {}", dancer.name)} />
            <div class="name-and-stats-container">
            <p>{&dancer.name}</p>
                <StatBar value={dancer.strength} label="strength" />
                <StatBar value={dancer.flexibility} label="flexibility" />
            </div>
        </div>
    }
}
