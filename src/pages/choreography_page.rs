//src/pages/choreography_page.rs
use crate::Route;
use crate::components::molecules::video_list::{ChoreographyEntry, VideoList};
use yew::prelude::*;
use yew_router::prelude::use_navigator;

const DRAFT_CHOREOGRAPHY_COUNT: u32 = 4;
pub(crate) const DRAFT_CHOREOGRAPHIES_STORAGE_KEY: &str = "draft_choreographies";
const CONFIRMED_CHOREOGRAPHIES_STORAGE_KEY: &str = "confirmed_choreographies";

fn load_choreographies(key: &str) -> Option<Vec<ChoreographyEntry>> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(key).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
}

fn save_choreographies(key: &str, entries: &[ChoreographyEntry]) {
    if let Ok(json) = serde_json::to_string(entries) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(key, &json);
        }
    }
}

#[function_component(ChoreographyPage)]
pub fn choreography_page() -> Html {
    // Draft/confirmed lists are persisted to localStorage so edits survive a refresh
    let draft_choreographies = use_state(|| {
        load_choreographies(DRAFT_CHOREOGRAPHIES_STORAGE_KEY).unwrap_or_else(|| {
            (1..=DRAFT_CHOREOGRAPHY_COUNT)
                .map(ChoreographyEntry::new)
                .collect::<Vec<_>>()
        })
    });
    let confirmed_choreographies = use_state(|| {
        load_choreographies(CONFIRMED_CHOREOGRAPHIES_STORAGE_KEY).unwrap_or_default()
    });

    {
        let draft_choreographies = draft_choreographies.clone();
        use_effect_with(draft_choreographies.clone(), move |draft_choreographies| {
            save_choreographies(DRAFT_CHOREOGRAPHIES_STORAGE_KEY, &draft_choreographies);
            || ()
        });
    }

    {
        let confirmed_choreographies = confirmed_choreographies.clone();
        use_effect_with(
            confirmed_choreographies.clone(),
            move |confirmed_choreographies| {
                save_choreographies(CONFIRMED_CHOREOGRAPHIES_STORAGE_KEY, &confirmed_choreographies);
                || ()
            },
        );
    }

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

    let navigator = use_navigator().unwrap();
    let on_add_info = Callback::from(move |number: u32| {
        navigator.push(&Route::InfoPage { number });
    });

    html! {
        <div class="page about-choreo-container">
                <h2>{ "Choreography Page" }</h2>

                <VideoList
                    entries={(*draft_choreographies).clone()}
                    on_thumbnail_change={on_thumbnail_change}
                    on_title_change={on_title_change}
                    on_duration_change={on_duration_change}
                    on_checkout={on_checkout}
                    on_add_info={on_add_info}
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
