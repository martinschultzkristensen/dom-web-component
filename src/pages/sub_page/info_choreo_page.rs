//src/pages/sub_page/info_choreo_page.rs
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InfoPageProps {
    pub number: u32,
}

#[function_component(InfoPage)]
pub fn info_page(props: &InfoPageProps) -> Html {
    html! {
        <div class="page about-choreo-container">
                <h2>{ format!("Info side til koreografi NR. {}", props.number) }</h2>
                <p>{ "Her kan du tilføje informationer til de enkelte koreografier." }</p>
        </div>
    }
}
