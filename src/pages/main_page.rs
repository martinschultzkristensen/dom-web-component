use yew::prelude::*;

#[function_component(MainPage)]
pub fn main_page() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <div>
        <p>{ "THIS PAGES IS CALLED FROM MAIN PAGES!!!!" }</p>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
        </div>
    }
}