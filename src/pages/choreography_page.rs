//src/pages/choreography_page.rs
use crate::components::molecules::video_list::{ChoreographyEntry, VideoList};
use yew::prelude::*;

#[function_component(ChoreographyPage)]
pub fn choreography_page() -> Html {
    const MAX_DRAFT_CHOREOGRAPHIES: usize = 4;

    let draft_choreographies = use_state(Vec::<ChoreographyEntry>::new);
    let confirmed_choreographies = use_state(Vec::<ChoreographyEntry>::new);
    let next_number = use_state(|| 1u32);
    let max_reached_error = use_state(|| false);

    let on_add_choreography = {
        let draft_choreographies = draft_choreographies.clone();
        let next_number = next_number.clone();
        let max_reached_error = max_reached_error.clone();
        Callback::from(move |_| {
            if draft_choreographies.len() >= MAX_DRAFT_CHOREOGRAPHIES {
                max_reached_error.set(true);
                return;
            }

            let mut updated = (*draft_choreographies).clone();
            updated.push(ChoreographyEntry::new(*next_number));
            draft_choreographies.set(updated);
            next_number.set(*next_number + 1);
            max_reached_error.set(false);
        })
    };

    let on_thumbnail_change = {
        let draft_choreographies = draft_choreographies.clone();
        Callback::from(move |(number, data_url): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();
            if let Some(entry) = updated.iter_mut().find(|e| e.number == number) {
                entry.video_thumbnail = Some(data_url);
            }
            draft_choreographies.set(updated);
        })
    };

    let on_title_change = {
        let draft_choreographies = draft_choreographies.clone();
        Callback::from(move |(number, title): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();
            if let Some(entry) = updated.iter_mut().find(|e| e.number == number) {
                entry.title = title;
            }
            draft_choreographies.set(updated);
        })
    };

    let on_duration_change = {
        let draft_choreographies = draft_choreographies.clone();
        Callback::from(move |(number, duration): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();
            if let Some(entry) = updated.iter_mut().find(|e| e.number == number) {
                entry.duration = duration;
            }
            draft_choreographies.set(updated);
        })
    };

    let on_checkout = {
        let draft_choreographies = draft_choreographies.clone();
        let confirmed_choreographies = confirmed_choreographies.clone();
        Callback::from(move |number: u32| {
            let mut drafts = (*draft_choreographies).clone();
            if let Some(pos) = drafts.iter().position(|e| e.number == number) {
                let entry = drafts.remove(pos);
                let mut confirmed = (*confirmed_choreographies).clone();
                confirmed.push(entry);
                confirmed_choreographies.set(confirmed);
            }
            draft_choreographies.set(drafts);
        })
    };

    html! {
        <div class="page about-choreo-container">
                <h2>{ "Choreography Page" }</h2>

                <button class="main-action-button" onclick={on_add_choreography}>
                    { "+ tilføj dans" }
                </button>

                if *max_reached_error {
                    <p class="error-message">{ "Du har nået maks antal Koreografier" }</p>
                }

                <VideoList
                    entries={(*draft_choreographies).clone()}
                    on_thumbnail_change={on_thumbnail_change}
                    on_title_change={on_title_change}
                    on_duration_change={on_duration_change}
                    on_checkout={on_checkout}
                />

                if !confirmed_choreographies.is_empty() {
                    <h2>{ "Confirmed" }</h2>
                    <ul class="confirmed-choreography-list">
                        { for confirmed_choreographies.iter().map(|entry| html! {
                            <li key={entry.number}>{ format!("{}. {}", entry.number, entry.title) }</li>
                        }) }
                    </ul>
                }
        </div>
    }
}
