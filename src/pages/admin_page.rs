use crate::services::supabase::{
    create_choreography_file_signed_url, fetch_admin_pending_choreographies, get_my_profile,
    update_choreography_status, AdminChoreographyRow,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct AdminChoreographyView {
    id: String,
    title: String,
    duration_seconds: i32,
    description: String,
    image_url: Option<String>,
    demo_video_url: Option<String>,
    choreo_video_url: Option<String>,
    machine_id: String,
    dancers: Vec<String>,
}

fn machine_label(machine_id: &str) -> String {
    match machine_id {
        "machine_1" => "DanceOmatic 1".to_string(),
        "machine_2" => "DanceOmatic 2".to_string(),
        "machine_3" => "DanceOmatic 3".to_string(),
        "No machine selected" => "No machine selected".to_string(),
        other => other.to_string(),
    }
}

fn format_duration(total_seconds: i32) -> String {
    let safe_seconds = total_seconds.max(0);
    let minutes = safe_seconds / 60;
    let seconds = safe_seconds % 60;

    format!("{}:{:02}", minutes, seconds)
}

async fn row_to_view(row: AdminChoreographyRow) -> AdminChoreographyView {
    let image_url = create_choreography_file_signed_url(&row.image_path).await.ok();
    let demo_video_url = create_choreography_file_signed_url(&row.demo_video_path).await.ok();
    let choreo_video_url = create_choreography_file_signed_url(&row.choreo_video_path).await.ok();

    let machine_id = row
        .choreography_machines
        .first()
        .map(|machine| machine.machine_id.clone())
        .unwrap_or_else(|| "No machine selected".to_string());

    let mut dancer_links = row.choreography_dancers.clone();
    dancer_links.sort_by_key(|link| link.sort_order);

    let dancers = dancer_links
        .into_iter()
        .filter_map(|link| link.dancers.map(|dancer| dancer.name))
        .collect::<Vec<String>>();

    AdminChoreographyView {
        id: row.id,
        title: row.title,
        duration_seconds: row.duration_seconds,
        description: row.description,
        image_url,
        demo_video_url,
        choreo_video_url,
        machine_id,
        dancers,
    }
}

#[function_component(AdminPage)]
pub fn admin_page() -> Html {
    let is_loading_access = use_state(|| true);
    let role = use_state(|| None::<String>);
    let page_error = use_state(|| None::<String>);

    let is_loading_choreographies = use_state(|| false);
    let choreographies = use_state(Vec::<AdminChoreographyView>::new);
    let reload_counter = use_state(|| 0u32);
    let action_message = use_state(|| None::<String>);

    {
        let is_loading_access = is_loading_access.clone();
        let role = role.clone();
        let page_error = page_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                is_loading_access.set(true);

                match get_my_profile().await {
                    Ok(profile) => {
                        role.set(Some(profile.role));
                        page_error.set(None);
                    }
                    Err(message) => {
                        role.set(None);
                        page_error.set(Some(message));
                    }
                }

                is_loading_access.set(false);
            });

            || ()
        });
    }

        {
        let is_loading_choreographies = is_loading_choreographies.clone();
        let choreographies = choreographies.clone();
        let page_error = page_error.clone();

        let role_dependency = (*role).clone();
        let reload_dependency = *reload_counter;

        use_effect_with((role_dependency, reload_dependency), move |(role_value, _)| {
            if role_value.as_deref() == Some("admin") {
                spawn_local(async move {
                    is_loading_choreographies.set(true);

                    match fetch_admin_pending_choreographies().await {
                        Ok(rows) => {
                            let mut views = Vec::<AdminChoreographyView>::new();

                            for row in rows {
                                views.push(row_to_view(row).await);
                            }

                            choreographies.set(views);
                            page_error.set(None);
                        }
                        Err(message) => {
                            page_error.set(Some(message));
                        }
                    }

                    is_loading_choreographies.set(false);
                });
            }

            || ()
        });
    }

    let content = if *is_loading_access {
        html! {
            <p class="login-help-text">
                { "Loading admin access..." }
            </p>
        }
    } else if let Some(message) = &*page_error {
        html! {
            <p class="error-message">
                { message.clone() }
            </p>
        }
    } else if (*role).as_deref() != Some("admin") {
        html! {
            <div class="creator-help-box">
                <p>{ "You do not have access to this page." }</p>
            </div>
        }
    } else {
        let choreography_content = if *is_loading_choreographies {
            html! {
                <p class="login-help-text">
                    { "Loading pending choreographies..." }
                </p>
            }
        } else if (*choreographies).is_empty() {
            html! {
                <div class="creator-help-box">
                    <p>{ "There are no pending choreographies right now." }</p>
                </div>
            }
        } else {
            html! {
                <div class="admin-review-list">
                    {
                        for (*choreographies).iter().enumerate().map(|(index, item)| {
                            let display_number = index + 1;
                            let approve_id = item.id.clone();
                            let reject_id = item.id.clone();

                            let approve_title = item.title.clone();
                            let reject_title = item.title.clone();

                            let reload_counter_for_approve = reload_counter.clone();
                            let reload_counter_for_reject = reload_counter.clone();

                            let action_message_for_approve = action_message.clone();
                            let action_message_for_reject = action_message.clone();

                            let page_error_for_approve = page_error.clone();
                            let page_error_for_reject = page_error.clone();

                            let current_reload_for_approve = *reload_counter;
                            let current_reload_for_reject = *reload_counter;

                            let on_approve = Callback::from(move |_| {
                                let approve_id = approve_id.clone();
                                let approve_title = approve_title.clone();
                                let reload_counter = reload_counter_for_approve.clone();
                                let action_message = action_message_for_approve.clone();
                                let page_error = page_error_for_approve.clone();

                                spawn_local(async move {
                                    match update_choreography_status(
                                        approve_id,
                                        "approved".to_string(),
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            action_message.set(Some(format!(
                                                "Approved: {}",
                                                approve_title
                                            )));
                                            page_error.set(None);
                                            reload_counter.set(current_reload_for_approve + 1);
                                        }
                                        Err(message) => {
                                            page_error.set(Some(message));
                                        }
                                    }
                                });
                            });

                            let on_reject = Callback::from(move |_| {
                                let reject_id = reject_id.clone();
                                let reject_title = reject_title.clone();
                                let reload_counter = reload_counter_for_reject.clone();
                                let action_message = action_message_for_reject.clone();
                                let page_error = page_error_for_reject.clone();

                                spawn_local(async move {
                                    match update_choreography_status(
                                        reject_id,
                                        "rejected".to_string(),
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            action_message.set(Some(format!(
                                                "Rejected: {}",
                                                reject_title
                                            )));
                                            page_error.set(None);
                                            reload_counter.set(current_reload_for_reject + 1);
                                        }
                                        Err(message) => {
                                            page_error.set(Some(message));
                                        }
                                    }
                                });
                            });

                            let image_block = if let Some(image_url) = &item.image_url {
                                html! {
                                    <img
                                        class="admin-review-image"
                                        src={image_url.clone()}
                                        alt={item.title.clone()}
                                    />
                                }
                            } else {
                                html! {
                                    <div class="admin-review-image-placeholder">
                                        { "Image unavailable" }
                                    </div>
                                }
                            };

                            let demo_video_block = if let Some(demo_video_url) = &item.demo_video_url {
                                html! {
                                    <video
                                        class="admin-review-video"
                                        controls=true
                                        src={demo_video_url.clone()}
                                    />
                                }
                            } else {
                                html! {
                                    <div class="admin-review-video-placeholder">
                                        { "Demo video unavailable" }
                                    </div>
                                }
                            };

                            let choreo_video_block = if let Some(choreo_video_url) = &item.choreo_video_url {
                                html! {
                                    <video
                                        class="admin-review-video"
                                        controls=true
                                        src={choreo_video_url.clone()}
                                    />
                                }
                            } else {
                                html! {
                                    <div class="admin-review-video-placeholder">
                                        { "Choreography video unavailable" }
                                    </div>
                                }
                            };

                            html! {
                                <article class="admin-review-card" key={item.id.clone()}>
                                    <div class="admin-review-number">
                                        { format!("Pending #{}", display_number) }
                                    </div>

                                    <div class="admin-review-main">
                                        { image_block }

                                        <div class="admin-review-info">
                                            <h3>{ format!("Pending #{} - {}", display_number, item.title) }</h3>

                                            <p>
                                                <strong>{ "Duration: " }</strong>
                                                { format_duration(item.duration_seconds) }
                                            </p>

                                            <p>
                                                <strong>{ "Machine: " }</strong>
                                                { machine_label(&item.machine_id) }
                                            </p>

                                            <p>
                                                <strong>{ "Description: " }</strong>
                                                { item.description.clone() }
                                            </p>

                                            <div>
                                                <strong>{ "Dancers:" }</strong>

                                                if item.dancers.is_empty() {
                                                    <p>{ "No dancers found." }</p>
                                                } else {
                                                    <ul class="admin-dancer-list">
                                                        {
                                                            for item.dancers.iter().map(|dancer_name| {
                                                                html! {
                                                                    <li>{ dancer_name.clone() }</li>
                                                                }
                                                            })
                                                        }
                                                    </ul>
                                                }
                                            </div>

                                            <div class="admin-review-actions">
                                                <button
                                                    class="admin-approve-button"
                                                    onclick={on_approve}
                                                >
                                                    { "Approve" }
                                                </button>

                                                <button
                                                    class="admin-reject-button"
                                                    onclick={on_reject}
                                                >
                                                    { "Reject" }
                                                </button>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="admin-review-video-grid">
                                        <div>
                                            <h4>{ "Demo video" }</h4>
                                            { demo_video_block }
                                        </div>

                                        <div>
                                            <h4>{ "Choreography video" }</h4>
                                            { choreo_video_block }
                                        </div>
                                    </div>
                                </article>
                            }
                        })
                    }
                </div>
            }
        };

        html! {
            <>
                <div class="creator-help-box">
                    <p>
                        { "Review submitted choreographies before they become available on DanceOmatic machines." }
                    </p>
                </div>

                if let Some(message) = &*action_message {
                    <p class="admin-action-message">
                        { message.clone() }
                    </p>
                }

                <h2>{ "Pending choreographies" }</h2>

                { choreography_content }
            </>
        }
    };

    html! {
        <div class="page about-choreo-container">
            <h2>{ "Admin Page" }</h2>
            { content }
        </div>
    }
}