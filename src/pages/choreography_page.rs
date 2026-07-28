use crate::components::molecules::video_list::{ChoreographyEntry, VideoList};
use crate::services::supabase::{
    create_choreography_file_signed_url, fetch_submitted_choreographies, SubmittedChoreographyRow,
};
use crate::Route;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

// New v2 key so old hardcoded prototype data does not come back from localStorage.
pub(crate) const DRAFT_CHOREOGRAPHIES_STORAGE_KEY: &str = "draft_choreographies_v2";

#[derive(Clone, PartialEq)]
struct SubmittedChoreographyView {
    id: String,
    title: String,
    duration_seconds: i32,
    image_url: Option<String>,
    status: String,
    machine_id: Option<String>,
}

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

fn choreography_info_storage_key(number: u32) -> String {
    format!("choreo_info_{number}")
}

fn remove_choreography_info(number: u32) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.remove_item(&choreography_info_storage_key(number));
    }
}

fn renumber_choreography_drafts(entries: Vec<ChoreographyEntry>) -> Vec<ChoreographyEntry> {
    let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) else {
        return entries
            .into_iter()
            .enumerate()
            .map(|(index, mut entry)| {
                entry.number = index as u32 + 1;
                entry
            })
            .collect();
    };

    let old_info = entries
        .iter()
        .map(|entry| {
            let json = storage
                .get_item(&choreography_info_storage_key(entry.number))
                .ok()
                .flatten();

            (entry.number, json)
        })
        .collect::<Vec<(u32, Option<String>)>>();

    for (old_number, _) in &old_info {
        let _ = storage.remove_item(&choreography_info_storage_key(*old_number));
    }

    entries
        .into_iter()
        .enumerate()
        .map(|(index, mut entry)| {
            let old_number = entry.number;
            let new_number = index as u32 + 1;

            if let Some((_, Some(info_json))) =
                old_info.iter().find(|(number, _)| *number == old_number)
            {
                let _ = storage.set_item(&choreography_info_storage_key(new_number), info_json);
            }

            entry.number = new_number;
            entry
        })
        .collect()
}

fn next_choreography_number(entries: &[ChoreographyEntry]) -> u32 {
    entries
        .iter()
        .map(|entry| entry.number)
        .max()
        .unwrap_or(0)
        + 1
}

fn format_duration_seconds(duration_seconds: i32) -> String {
    let minutes = duration_seconds / 60;
    let seconds = duration_seconds % 60;

    format!("{}:{:02}", minutes, seconds)
}

fn machine_label(machine_id: &Option<String>) -> String {
    match machine_id.as_deref() {
        Some("machine_1") => "DanceOmatic 1".to_string(),
        Some("machine_2") => "DanceOmatic 2".to_string(),
        Some("machine_3") => "DanceOmatic 3".to_string(),
        _ => "DanceOmatic".to_string(),
    }
}

fn status_label(status: &str) -> String {
    match status {
        "pending" => "Waiting for admin review".to_string(),
        "approved" => "Approved".to_string(),
        "rejected" => "Rejected".to_string(),
        _ => status.to_string(),
    }
}

async fn submitted_row_to_view(
    row: SubmittedChoreographyRow,
) -> SubmittedChoreographyView {
    let image_url = create_choreography_file_signed_url(&row.image_path)
        .await
        .ok();

    let machine_id = row
        .choreography_machines
        .first()
        .map(|machine| machine.machine_id.clone());

    SubmittedChoreographyView {
        id: row.id,
        title: row.title,
        duration_seconds: row.duration_seconds,
        image_url,
        status: row.status,
        machine_id,
    }
}

#[function_component(ChoreographyPage)]
pub fn choreography_page() -> Html {
    let draft_choreographies = use_state(|| {
        load_choreographies(DRAFT_CHOREOGRAPHIES_STORAGE_KEY).unwrap_or_default()
    });

    let submitted_choreographies = use_state(Vec::<SubmittedChoreographyView>::new);
    let submitted_error = use_state(|| None::<String>);
    let is_loading_submitted = use_state(|| true);

    {
        let draft_choreographies = draft_choreographies.clone();

        use_effect_with(draft_choreographies.clone(), move |draft_choreographies| {
            save_choreographies(DRAFT_CHOREOGRAPHIES_STORAGE_KEY, &draft_choreographies);
            || ()
        });
    }

    {
        let submitted_choreographies = submitted_choreographies.clone();
        let submitted_error = submitted_error.clone();
        let is_loading_submitted = is_loading_submitted.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                is_loading_submitted.set(true);

                match fetch_submitted_choreographies().await {
                    Ok(rows) => {
                        let mut views = Vec::<SubmittedChoreographyView>::new();

                        for row in rows {
                            views.push(submitted_row_to_view(row).await);
                        }

                        submitted_choreographies.set(views);
                        submitted_error.set(None);
                    }
                    Err(message) => {
                        submitted_choreographies.set(Vec::new());
                        submitted_error.set(Some(message));
                    }
                }

                is_loading_submitted.set(false);
            });

            || ()
        });
    }

    let on_add_choreography = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |_| {
            let mut updated = (*draft_choreographies).clone();
            let next_number = next_choreography_number(&updated);

            updated.push(ChoreographyEntry::new(next_number));
            draft_choreographies.set(updated);
        })
    };

    let on_remove_choreography = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |number: u32| {
            let should_remove = web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message(&format!("Remove choreography No. {}?", number))
                        .ok()
                })
                .unwrap_or(false);

            if !should_remove {
                return;
            }

            remove_choreography_info(number);

            let updated = (*draft_choreographies)
                .clone()
                .into_iter()
                .filter(|entry| entry.number != number)
                .collect::<Vec<ChoreographyEntry>>();

            let renumbered = renumber_choreography_drafts(updated);

            draft_choreographies.set(renumbered);
        })
    };

    let on_thumbnail_change = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |(number, data_url): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();

            if let Some(entry) = updated.iter_mut().find(|entry| entry.number == number) {
                entry.video_thumbnail = Some(data_url);
            }

            draft_choreographies.set(updated);
        })
    };

    let on_demo_video_path_change = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |(number, path): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();

            if let Some(entry) = updated.iter_mut().find(|entry| entry.number == number) {
                entry.demo_video_path = Some(path);
            }

            draft_choreographies.set(updated);
        })
    };

    let on_title_change = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |(number, title): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();

            if let Some(entry) = updated.iter_mut().find(|entry| entry.number == number) {
                entry.title = title;
            }

            draft_choreographies.set(updated);
        })
    };

    let on_duration_change = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |(number, duration): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();

            if let Some(entry) = updated.iter_mut().find(|entry| entry.number == number) {
                entry.duration = duration;
            }

            draft_choreographies.set(updated);
        })
    };

    let on_machine_change = {
        let draft_choreographies = draft_choreographies.clone();

        Callback::from(move |(number, machine): (u32, String)| {
            let mut updated = (*draft_choreographies).clone();

            if let Some(entry) = updated.iter_mut().find(|entry| entry.number == number) {
                entry.target_machine = machine;
            }

            draft_choreographies.set(updated);
        })
    };

    let navigator = use_navigator().unwrap();

    let on_add_info = {
        let navigator = navigator.clone();

        Callback::from(move |number: u32| {
            navigator.push(&Route::InfoPage { number });
        })
    };

    html! {
        <div class="page about-choreo-container">
            <h2>{ "Choreography Page" }</h2>

            <div class="creator-help-box">
                <p>
                    { "Create a choreography draft, upload a demo video, enter a title and duration, and choose the DanceOmatic machine that should receive it." }
                </p>
                <p>
                    { "Open Details to add the choreography image, description, choreography video and dancers. All required fields must be completed before the choreography can be sent." }
                </p>
            </div>

            <div class="main-panel">
                <button
                    type="button"
                    class="main-action-button"
                    onclick={on_add_choreography}
                >
                    { "+ Add Choreography" }
                </button>
            </div>

            <h2>{ "Draft choreographies" }</h2>

            if draft_choreographies.is_empty() {
                <p class="login-help-text">
                    { "No draft choreographies. Click + Add Choreography to create a new draft." }
                </p>
            } else {
                <VideoList
                    entries={(*draft_choreographies).clone()}
                    on_thumbnail_change={on_thumbnail_change}
                    on_demo_video_path_change={on_demo_video_path_change}
                    on_title_change={on_title_change}
                    on_duration_change={on_duration_change}
                    on_machine_change={on_machine_change}
                    on_add_info={on_add_info}
                    on_remove={on_remove_choreography}
                />
            }

            <div class="submitted-choreographies-section">
                <h2>{ "Submitted choreographies" }</h2>

                if *is_loading_submitted {
                    <p class="login-help-text">
                        { "Loading submitted choreographies..." }
                    </p>
                }

                if let Some(message) = &*submitted_error {
                    <p class="error-message">
                        { message }
                    </p>
                }

                if !*is_loading_submitted && submitted_choreographies.is_empty() && submitted_error.is_none() {
                    <p class="login-help-text">
                        { "No submitted choreographies yet." }
                    </p>
                }

                if !submitted_choreographies.is_empty() {
                    <div class="submitted-choreographies-grid">
                        {
                            for submitted_choreographies.iter().map(|choreography| {
                                html! {
                                    <div key={choreography.id.clone()} class="submitted-choreography-card">
                                        if let Some(image_url) = &choreography.image_url {
                                            <img
                                                src={image_url.clone()}
                                                alt="Submitted choreography"
                                                class="submitted-choreography-image"
                                            />
                                        } else {
                                            <div class="submitted-choreography-image-placeholder">
                                                { "Image preview unavailable" }
                                            </div>
                                        }

                                        <div class="submitted-choreography-content">
                                            <h3>{ choreography.title.clone() }</h3>

                                            <p>
                                                { format!("Duration: {}", format_duration_seconds(choreography.duration_seconds)) }
                                            </p>

                                            <p>
                                                { format!("Machine: {}", machine_label(&choreography.machine_id)) }
                                            </p>

                                            <span class="submitted-status-pill">
                                                { status_label(&choreography.status) }
                                            </span>
                                        </div>
                                    </div>
                                }
                            })
                        }
                    </div>
                }
            </div>
        </div>
    }
}