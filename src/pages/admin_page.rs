use crate::services::supabase::{
    create_choreography_file_signed_url, fetch_admin_machine_delivery_workspace,
    fetch_admin_pending_choreographies, fetch_machine_media, get_my_profile,
    replace_admin_machine_draft, send_admin_machine_draft, update_choreography_status,
    update_machine_media, upload_machine_media_video, AdminChoreographyRow,
    AdminMachineDeliveryWorkspace, MachineMediaRow,
};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, File, HtmlInputElement, Url};
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

#[derive(Clone, PartialEq)]
struct AdminMachineMediaView {
    machine_id: String,
    intro_video_path: Option<String>,
    load_video_path: Option<String>,
    intro_video_url: Option<String>,
    load_video_url: Option<String>,
    updated_at: String,
}

fn has_media_path(path: &Option<String>) -> bool {
    path.as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

async fn machine_media_row_to_view(row: MachineMediaRow) -> AdminMachineMediaView {
    let intro_video_url = if has_media_path(&row.intro_video_path) {
        create_choreography_file_signed_url(row.intro_video_path.as_deref().unwrap())
            .await
            .ok()
    } else {
        None
    };

    let load_video_url = if has_media_path(&row.load_video_path) {
        create_choreography_file_signed_url(row.load_video_path.as_deref().unwrap())
            .await
            .ok()
    } else {
        None
    };

    AdminMachineMediaView {
        machine_id: row.machine_id,
        intro_video_path: row.intro_video_path,
        load_video_path: row.load_video_path,
        intro_video_url,
        load_video_url,
        updated_at: row.updated_at,
    }
}

fn selected_file_from_event(event: Event) -> Option<File> {
    event
        .target()
        .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
        .and_then(|input| input.files())
        .and_then(|files| files.get(0))
}

fn selected_value_from_event(event: Event) -> String {
    event
        .target()
        .and_then(|target| js_sys::Reflect::get(&target, &JsValue::from_str("value")).ok())
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "machine_1".to_string())
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
        dancers,
    }
}

#[derive(Properties, PartialEq)]
struct AdminMachineDeliveryPanelProps {
    machine_id: String,
    approved_library_reload: u32,
    on_action_message: Callback<String>,
}

#[function_component(AdminMachineDeliveryPanel)]
fn admin_machine_delivery_panel(props: &AdminMachineDeliveryPanelProps) -> Html {
    let workspace = use_state(|| None::<AdminMachineDeliveryWorkspace>);
    let is_loading = use_state(|| true);
    let is_saving_draft = use_state(|| false);
    let is_sending = use_state(|| false);
    let error = use_state(|| None::<String>);
    let local_reload_counter = use_state(|| 0u32);

    {
        let workspace = workspace.clone();
        let is_loading = is_loading.clone();
        let error = error.clone();

        let machine_id_dependency = props.machine_id.clone();
        let approved_library_reload_dependency = props.approved_library_reload;
        let local_reload_dependency = *local_reload_counter;

        use_effect_with(
            (
                machine_id_dependency,
                approved_library_reload_dependency,
                local_reload_dependency,
            ),
            move |(machine_id, _, _)| {
                let machine_id = machine_id.clone();

                spawn_local(async move {
                    is_loading.set(true);

                    match fetch_admin_machine_delivery_workspace(&machine_id).await {
                        Ok(result) => {
                            workspace.set(Some(result));
                            error.set(None);
                        }
                        Err(message) => {
                            workspace.set(None);
                            error.set(Some(message));
                        }
                    }

                    is_loading.set(false);
                });

                || ()
            },
        );
    }

    let on_save_draft = {
        let workspace = workspace.clone();
        let is_saving_draft = is_saving_draft.clone();
        let is_sending = is_sending.clone();
        let error = error.clone();
        let machine_id = props.machine_id.clone();

        Callback::from(move |choreography_ids: Vec<String>| {
            if *is_saving_draft || *is_sending {
                return;
            }

            let workspace = workspace.clone();
            let is_saving_draft_for_task = is_saving_draft.clone();
            let error = error.clone();
            let machine_id = machine_id.clone();

            is_saving_draft.set(true);
            error.set(None);

            spawn_local(async move {
                match replace_admin_machine_draft(&machine_id, choreography_ids).await {
                    Ok(updated_workspace) => {
                        workspace.set(Some(updated_workspace));
                        error.set(None);
                    }
                    Err(message) => {
                        error.set(Some(message));
                    }
                }

                is_saving_draft_for_task.set(false);
            });
        })
    };

    let on_send = {
        let is_saving_draft = is_saving_draft.clone();
        let is_sending = is_sending.clone();
        let error = error.clone();
        let local_reload_counter = local_reload_counter.clone();
        let on_action_message = props.on_action_message.clone();
        let machine_id = props.machine_id.clone();
        let current_local_reload = *local_reload_counter;

        Callback::from(move |_| {
            if *is_saving_draft || *is_sending {
                return;
            }

            let should_send = web_sys::window()
                .and_then(|window| {
                    window
                        .confirm_with_message(
                            "Send this complete draft to the selected DanceOmatic machine?",
                        )
                        .ok()
                })
                .unwrap_or(false);

            if !should_send {
                return;
            }

            let is_sending_for_task = is_sending.clone();
            let error = error.clone();
            let local_reload_counter = local_reload_counter.clone();
            let on_action_message = on_action_message.clone();
            let machine_id = machine_id.clone();

            is_sending.set(true);
            error.set(None);

            spawn_local(async move {
                match send_admin_machine_draft(&machine_id).await {
                    Ok(result) => {
                        on_action_message.emit(format!(
                            "Sent {} version {} with {} choreographies and {} files.",
                            result.machine_display_name,
                            result.version,
                            result.choreography_count,
                            result.file_count
                        ));
                        error.set(None);
                        local_reload_counter.set(current_local_reload + 1);
                    }
                    Err(message) => {
                        error.set(Some(message));
                    }
                }

                is_sending_for_task.set(false);
            });
        })
    };

    let is_busy = *is_saving_draft || *is_sending;

    let panel_content = if *is_loading {
        html! {
            <p class="login-help-text">
                { "Loading machine choreography delivery..." }
            </p>
        }
    } else if let Some(workspace_value) = &*workspace {
        let draft_ids = workspace_value
            .draft
            .iter()
            .map(|item| item.choreography_id.clone())
            .collect::<Vec<String>>();

        let available_library_count = workspace_value
            .approved_library
            .iter()
            .filter(|item| !item.selected)
            .count();

        let latest_deployment = if let Some(deployment) = &workspace_value.latest_deployment {
            html! {
                <div class="creator-help-box">
                    <p>
                        <strong>{ "Latest sent version: " }</strong>
                        { deployment.version.to_string() }
                    </p>
                    <p>
                        <strong>{ "Status: " }</strong>
                        { deployment.status.clone() }
                    </p>
                    <p>
                        <strong>{ "Content: " }</strong>
                        {
                            format!(
                                "{} choreographies / {} files",
                                deployment.choreography_count,
                                deployment.file_count
                            )
                        }
                    </p>
                    <p>
                        <strong>{ "Created: " }</strong>
                        { deployment.created_at.clone() }
                    </p>

                    if let Some(last_error) = &deployment.last_error {
                        <p class="error-message">
                            { last_error.clone() }
                        </p>
                    }
                </div>
            }
        } else {
            html! {
                <div class="creator-help-box">
                    <p>{ "No version has been sent to this machine yet." }</p>
                </div>
            }
        };

        html! {
            <>
                <div class="machine-media-header">
                    <div>
                        <h2>{ "Machine choreography delivery" }</h2>
                        <p>
                            {
                                "Build the exact ordered choreography menu for the selected machine. Draft changes do not affect the physical machine until Send is pressed."
                            }
                        </p>
                    </div>
                </div>

                <p class="machine-media-updated">
                    <strong>{ "Selected target machine: " }</strong>
                    { workspace_value.machine.display_name.clone() }
                </p>

                if !workspace_value.machine.is_active {
                    <p class="error-message">
                        { "This machine is disabled and cannot receive new content." }
                    </p>
                }

                { latest_deployment }

                if let Some(message) = &*error {
                    <p class="error-message">
                        { message.clone() }
                    </p>
                }

                <div class="machine-media-grid">
                    <article class="machine-media-card">
                        <h3>{ "Approved library" }</h3>
                        <p class="machine-media-selected-file">
                            {
                                format!(
                                    "{} approved choreographies are available to add.",
                                    available_library_count
                                )
                            }
                        </p>

                        if workspace_value.approved_library.is_empty() {
                            <div class="creator-help-box">
                                <p>{ "There are no approved choreographies in the library yet." }</p>
                            </div>
                        } else if available_library_count == 0 {
                            <div class="creator-help-box">
                                <p>{ "Every approved choreography is already in this draft." }</p>
                            </div>
                        } else {
                            {
                                for workspace_value
                                    .approved_library
                                    .iter()
                                    .filter(|item| !item.selected)
                                    .map(|item| {
                                        let choreography_id = item.id.clone();
                                        let choreography_title = item.title.clone();
                                        let duration_seconds = item.duration_seconds;
                                        let current_ids = draft_ids.clone();
                                        let on_save_draft = on_save_draft.clone();

                                        let on_add = Callback::from(move |_| {
                                            let mut updated_ids = current_ids.clone();
                                            updated_ids.push(choreography_id.clone());
                                            on_save_draft.emit(updated_ids);
                                        });

                                        html! {
                                            <div class="creator-help-box" key={item.id.clone()}>
                                                <p>
                                                    <strong>{ choreography_title }</strong>
                                                </p>
                                                <p>
                                                    {
                                                        format!(
                                                            "Duration: {}",
                                                            format_duration(duration_seconds)
                                                        )
                                                    }
                                                </p>
                                                <button
                                                    class="admin-approve-button"
                                                    onclick={on_add}
                                                    disabled={is_busy}
                                                >
                                                    { "Add to machine draft" }
                                                </button>
                                            </div>
                                        }
                                    })
                            }
                        }
                    </article>

                    <article class="machine-media-card">
                        <h3>{ "Machine draft" }</h3>
                        <p class="machine-media-selected-file">
                            {
                                format!(
                                    "{} choreographies in the exact order shown below.",
                                    workspace_value.draft.len()
                                )
                            }
                        </p>

                        if workspace_value.draft.is_empty() {
                            <div class="creator-help-box">
                                <p>
                                    {
                                        "The draft is empty. Add at least one approved choreography before sending."
                                    }
                                </p>
                            </div>
                        } else {
                            {
                                for workspace_value.draft.iter().enumerate().map(|(index, item)| {
                                    let current_ids_for_up = draft_ids.clone();
                                    let current_ids_for_down = draft_ids.clone();
                                    let current_ids_for_remove = draft_ids.clone();

                                    let on_save_draft_for_up = on_save_draft.clone();
                                    let on_save_draft_for_down = on_save_draft.clone();
                                    let on_save_draft_for_remove = on_save_draft.clone();

                                    let on_move_up = Callback::from(move |_| {
                                        if index == 0 {
                                            return;
                                        }

                                        let mut updated_ids = current_ids_for_up.clone();
                                        updated_ids.swap(index, index - 1);
                                        on_save_draft_for_up.emit(updated_ids);
                                    });

                                    let on_move_down = Callback::from(move |_| {
                                        if index + 1 >= current_ids_for_down.len() {
                                            return;
                                        }

                                        let mut updated_ids = current_ids_for_down.clone();
                                        updated_ids.swap(index, index + 1);
                                        on_save_draft_for_down.emit(updated_ids);
                                    });

                                    let on_remove = Callback::from(move |_| {
                                        if index >= current_ids_for_remove.len() {
                                            return;
                                        }

                                        let mut updated_ids = current_ids_for_remove.clone();
                                        updated_ids.remove(index);
                                        on_save_draft_for_remove.emit(updated_ids);
                                    });

                                    html! {
                                        <div
                                            class="creator-help-box"
                                            key={item.choreography_id.clone()}
                                        >
                                            <p>
                                                <strong>
                                                    {
                                                        format!(
                                                            "{}. {}",
                                                            item.display_order,
                                                            item.title
                                                        )
                                                    }
                                                </strong>
                                            </p>
                                            <p>
                                                {
                                                    format!(
                                                        "Duration: {}",
                                                        format_duration(item.duration_seconds)
                                                    )
                                                }
                                            </p>

                                            <div class="admin-review-actions">
                                                <button
                                                    class="small-remove-button"
                                                    onclick={on_move_up}
                                                    disabled={is_busy || index == 0}
                                                >
                                                    { "Move up" }
                                                </button>

                                                <button
                                                    class="small-remove-button"
                                                    onclick={on_move_down}
                                                    disabled={
                                                        is_busy
                                                            || index + 1
                                                                >= workspace_value.draft.len()
                                                    }
                                                >
                                                    { "Move down" }
                                                </button>

                                                <button
                                                    class="admin-reject-button"
                                                    onclick={on_remove}
                                                    disabled={is_busy}
                                                >
                                                    { "Remove" }
                                                </button>
                                            </div>
                                        </div>
                                    }
                                })
                            }
                        }

                        <button
                            class="machine-media-save-button"
                            onclick={on_send}
                            disabled={
                                is_busy
                                    || workspace_value.draft.is_empty()
                                    || !workspace_value.machine.is_active
                            }
                        >
                            {
                                if *is_sending {
                                    "Sending..."
                                } else if *is_saving_draft {
                                    "Saving draft..."
                                } else {
                                    "Send to machine"
                                }
                            }
                        </button>
                    </article>
                </div>
            </>
        }
    } else if let Some(message) = &*error {
        html! {
            <p class="error-message">
                { message.clone() }
            </p>
        }
    } else {
        html! {
            <div class="creator-help-box">
                <p>{ "Machine delivery workspace is unavailable." }</p>
            </div>
        }
    };

    html! {
        <section class="machine-media-panel">
            { panel_content }
        </section>
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

    let is_loading_machine_media = use_state(|| false);
    let machine_media = use_state(Vec::<AdminMachineMediaView>::new);
    let machine_media_reload_counter = use_state(|| 0u32);
    let machine_media_error = use_state(|| None::<String>);
    let selected_machine_id = use_state(|| "machine_1".to_string());
    let selected_intro_file = use_state(|| None::<File>);
    let selected_load_file = use_state(|| None::<File>);
    let selected_intro_file_name = use_state(|| None::<String>);
    let selected_load_file_name = use_state(|| None::<String>);
    let selected_intro_preview_url = use_state(|| None::<String>);
    let selected_load_preview_url = use_state(|| None::<String>);
    let is_saving_machine_media = use_state(|| false);
    let machine_select_ref = use_node_ref();

    {
        let machine_select_ref = machine_select_ref.clone();
        let selected_machine_id_dependency = (*selected_machine_id).clone();
        let is_loading_machine_media_dependency = *is_loading_machine_media;

        use_effect_with(
            (selected_machine_id_dependency, is_loading_machine_media_dependency),
            move |(machine_id, _)| {
                if let Some(select_element) = machine_select_ref.cast::<web_sys::Element>() {
                    let _ = js_sys::Reflect::set(
                        select_element.as_ref(),
                        &JsValue::from_str("value"),
                        &JsValue::from_str(machine_id.as_str()),
                    );
                }

                || ()
            },
        );
    }

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

    {
        let is_loading_machine_media = is_loading_machine_media.clone();
        let machine_media = machine_media.clone();
        let machine_media_error = machine_media_error.clone();

        let role_dependency = (*role).clone();
        let reload_dependency = *machine_media_reload_counter;

        use_effect_with((role_dependency, reload_dependency), move |(role_value, _)| {
            if role_value.as_deref() == Some("admin") {
                spawn_local(async move {
                    is_loading_machine_media.set(true);

                    match fetch_machine_media().await {
                        Ok(rows) => {
                            let mut views = Vec::<AdminMachineMediaView>::new();

                            for row in rows {
                                views.push(machine_media_row_to_view(row).await);
                            }

                            machine_media.set(views);
                            machine_media_error.set(None);
                        }
                        Err(message) => {
                            machine_media_error.set(Some(message));
                        }
                    }

                    is_loading_machine_media.set(false);
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
        let selected_machine_id_value = (*selected_machine_id).clone();

        let selected_media = (*machine_media)
            .iter()
            .find(|media| media.machine_id == selected_machine_id_value)
            .cloned();

        let current_intro_path = selected_media
            .as_ref()
            .and_then(|media| media.intro_video_path.clone());

        let current_load_path = selected_media
            .as_ref()
            .and_then(|media| media.load_video_path.clone());

        let intro_preview_url = (*selected_intro_preview_url)
            .clone()
            .or_else(|| {
                selected_media
                    .as_ref()
                    .and_then(|media| media.intro_video_url.clone())
            });

        let intro_preview = if let Some(intro_url) = intro_preview_url {
            html! {
                <video
                    class="machine-media-preview-video"
                    controls=true
                    preload="metadata"
                    src={intro_url}
                />
            }
        } else {
            html! {
                <div class="machine-media-video-placeholder">
                    { "No intro video saved for this machine yet" }
                </div>
            }
        };

        let load_preview_url = (*selected_load_preview_url)
            .clone()
            .or_else(|| {
                selected_media
                    .as_ref()
                    .and_then(|media| media.load_video_url.clone())
            });

        let load_preview = if let Some(load_url) = load_preview_url {
            html! {
                <video
                class="machine-media-preview-video"
                controls=true
                preload="metadata"
                src={load_url}
            />
            }
        } else {
            html! {
                <div class="machine-media-video-placeholder">
                    { "No load video saved for this machine yet" }
                </div>
            }
        };

        let selected_updated_at = selected_media
            .as_ref()
            .map(|media| media.updated_at.clone())
            .unwrap_or_else(|| "Not updated yet".to_string());

        let on_machine_change = {
            let selected_machine_id = selected_machine_id.clone();
            let selected_intro_file = selected_intro_file.clone();
            let selected_load_file = selected_load_file.clone();
            let selected_intro_file_name = selected_intro_file_name.clone();
            let selected_load_file_name = selected_load_file_name.clone();
            let selected_intro_preview_url = selected_intro_preview_url.clone();
            let selected_load_preview_url = selected_load_preview_url.clone();
            let machine_media_error = machine_media_error.clone();

            Callback::from(move |event: Event| {
                selected_machine_id.set(selected_value_from_event(event));
                selected_intro_file.set(None);
                selected_load_file.set(None);
                selected_intro_file_name.set(None);
                selected_load_file_name.set(None);
                selected_intro_preview_url.set(None);
                selected_load_preview_url.set(None);
                machine_media_error.set(None);
            })
        };

        let on_intro_file_change = {
            let selected_intro_file = selected_intro_file.clone();
            let selected_intro_file_name = selected_intro_file_name.clone();
            let selected_intro_preview_url = selected_intro_preview_url.clone();
            let machine_media_error = machine_media_error.clone();

            Callback::from(move |event: Event| {
                if let Some(file) = selected_file_from_event(event) {
                    let file_name = file.name();
                    let preview_url = Url::create_object_url_with_blob(&file).ok();

                    selected_intro_file_name.set(Some(file_name));
                    selected_intro_preview_url.set(preview_url);
                    selected_intro_file.set(Some(file));
                    machine_media_error.set(None);
                }
            })
        };

        let on_load_file_change = {
            let selected_load_file = selected_load_file.clone();
            let selected_load_file_name = selected_load_file_name.clone();
            let selected_load_preview_url = selected_load_preview_url.clone();
            let machine_media_error = machine_media_error.clone();

            Callback::from(move |event: Event| {
                if let Some(file) = selected_file_from_event(event) {
                    let file_name = file.name();
                    let preview_url = Url::create_object_url_with_blob(&file).ok();

                    selected_load_file_name.set(Some(file_name));
                    selected_load_preview_url.set(preview_url);
                    selected_load_file.set(Some(file));
                    machine_media_error.set(None);
                }
            })
        };

        let on_save_machine_media = {
            let selected_intro_preview_url = selected_intro_preview_url.clone();
            let selected_load_preview_url = selected_load_preview_url.clone();
            let machine_id = (*selected_machine_id).clone();
            let existing_intro_path = current_intro_path.clone();
            let existing_load_path = current_load_path.clone();
            let selected_intro_file_value = (*selected_intro_file).clone();
            let selected_load_file_value = (*selected_load_file).clone();

            let selected_intro_file = selected_intro_file.clone();
            let selected_load_file = selected_load_file.clone();
            let selected_intro_file_name = selected_intro_file_name.clone();
            let selected_load_file_name = selected_load_file_name.clone();

            let is_saving_machine_media = is_saving_machine_media.clone();
            let machine_media_reload_counter = machine_media_reload_counter.clone();
            let action_message = action_message.clone();
            let machine_media_error = machine_media_error.clone();
            let current_machine_media_reload = *machine_media_reload_counter;

            Callback::from(move |_| {
                let selected_intro_preview_url = selected_intro_preview_url.clone();
                let selected_load_preview_url = selected_load_preview_url.clone();
                let machine_id = machine_id.clone();
                let existing_intro_path = existing_intro_path.clone();
                let existing_load_path = existing_load_path.clone();
                let selected_intro_file_value = selected_intro_file_value.clone();
                let selected_load_file_value = selected_load_file_value.clone();

                let selected_intro_file = selected_intro_file.clone();
                let selected_load_file = selected_load_file.clone();
                let selected_intro_file_name = selected_intro_file_name.clone();
                let selected_load_file_name = selected_load_file_name.clone();

                let is_saving_machine_media = is_saving_machine_media.clone();
                let machine_media_reload_counter = machine_media_reload_counter.clone();
                let action_message = action_message.clone();
                let machine_media_error = machine_media_error.clone();

                spawn_local(async move {
                    if selected_intro_file_value.is_none() && selected_load_file_value.is_none() {
                        machine_media_error.set(Some(
                            "Choose an intro video or load video before saving.".to_string(),
                        ));
                        return;
                    }

                    is_saving_machine_media.set(true);
                    machine_media_error.set(None);

                    let mut intro_path = existing_intro_path;
                    let mut load_path = existing_load_path;

                    if let Some(file) = selected_intro_file_value {
                        match upload_machine_media_video(file, &machine_id, "intro_video").await {
                            Ok(path) => intro_path = Some(path),
                            Err(message) => {
                                machine_media_error.set(Some(message));
                                is_saving_machine_media.set(false);
                                return;
                            }
                        }
                    }

                    if let Some(file) = selected_load_file_value {
                        match upload_machine_media_video(file, &machine_id, "load_video").await {
                            Ok(path) => load_path = Some(path),
                            Err(message) => {
                                machine_media_error.set(Some(message));
                                is_saving_machine_media.set(false);
                                return;
                            }
                        }
                    }

                    match update_machine_media(machine_id.clone(), intro_path, load_path).await {
                        Ok(_) => {
                            action_message.set(Some(format!(
                                "Updated machine media for {}",
                                machine_label(&machine_id)
                            )));
                            selected_intro_file.set(None);
                            selected_load_file.set(None);
                            selected_intro_preview_url.set(None);
                            selected_load_preview_url.set(None);
                            selected_intro_file_name.set(None);
                            selected_load_file_name.set(None);
                            machine_media_reload_counter.set(current_machine_media_reload + 1);
                        }
                        Err(message) => {
                            machine_media_error.set(Some(message));
                        }
                    }

                    is_saving_machine_media.set(false);
                });
            })
        };

        let machine_media_content = if *is_loading_machine_media {
            html! {
                <p class="login-help-text">
                    { "Loading machine media..." }
                </p>
            }
        } else {
            html! {
                <section class="machine-media-panel">
                    <div class="machine-media-header">
                        <div>
                            <h2>{ "Machine media" }</h2>
                            <p>
                                { "Upload and manage the default intro video and load video for each DanceOmatic machine." }
                            </p>
                        </div>

                        <label class="machine-media-select-label">
                            <span>{ "Select machine" }</span>
                            <select
                                ref={machine_select_ref.clone()}
                                class="machine-media-select"
                                autocomplete="off"
                                value={selected_machine_id_value.clone()}
                                onchange={on_machine_change}
                            >
                                <option value="machine_1">{ "DanceOmatic 1" }</option>
                                <option value="machine_2">{ "DanceOmatic 2" }</option>
                                <option value="machine_3">{ "DanceOmatic 3" }</option>
                            </select>
                        </label>
                    </div>

                    if let Some(message) = &*machine_media_error {
                        <p class="error-message">
                            { message.clone() }
                        </p>
                    }

                    <p class="machine-media-updated">
                        <strong>{ "Current machine: " }</strong>
                        { machine_label(&selected_machine_id_value) }
                        <br />
                        <strong>{ "Last updated: " }</strong>
                        { selected_updated_at }
                    </p>

                    <div class="machine-media-grid">
                        <article class="machine-media-card">
                            <h3>{ "Intro video" }</h3>
                            { intro_preview }

                            <p class="machine-media-path">
                                <strong>{ "Current path: " }</strong>
                                { current_intro_path.clone().unwrap_or_else(|| "Not set".to_string()) }
                            </p>

                            <label class="machine-media-upload-label">
                                <span>{ "Choose new intro video" }</span>
                                <input
                                    type="file"
                                    accept="video/*"
                                    onchange={on_intro_file_change}
                                />
                            </label>

                            <p class="machine-media-selected-file">
                                {
                                    if let Some(file_name) = &*selected_intro_file_name {
                                        format!("Selected: {}", file_name)
                                    } else {
                                        "No new intro video selected".to_string()
                                    }
                                }
                            </p>
                        </article>

                        <article class="machine-media-card">
                            <h3>{ "Load video" }</h3>
                            { load_preview }

                            <p class="machine-media-path">
                                <strong>{ "Current path: " }</strong>
                                { current_load_path.clone().unwrap_or_else(|| "Not set".to_string()) }
                            </p>

                            <label class="machine-media-upload-label">
                                <span>{ "Choose new load video" }</span>
                                <input
                                    type="file"
                                    accept="video/*"
                                    onchange={on_load_file_change}
                                />
                            </label>

                            <p class="machine-media-selected-file">
                                {
                                    if let Some(file_name) = &*selected_load_file_name {
                                        format!("Selected: {}", file_name)
                                    } else {
                                        "No new load video selected".to_string()
                                    }
                                }
                            </p>
                        </article>
                    </div>

                    <button
                        class="machine-media-save-button"
                        onclick={on_save_machine_media}
                        disabled={*is_saving_machine_media}
                    >
                        {
                            if *is_saving_machine_media {
                                "Saving..."
                            } else {
                                "Save machine media"
                            }
                        }
                    </button>
                </section>
            }
        };

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

        let on_machine_delivery_action = {
            let action_message = action_message.clone();

            Callback::from(move |message: String| {
                action_message.set(Some(message));
            })
        };

        html! {
            <>
                if let Some(message) = &*action_message {
                    <p class="admin-action-message">
                        { message.clone() }
                    </p>
                }

                { machine_media_content }

                <AdminMachineDeliveryPanel
                    key={selected_machine_id_value.clone()}
                    machine_id={selected_machine_id_value.clone()}
                    approved_library_reload={*reload_counter}
                    on_action_message={on_machine_delivery_action}
                />

                <section class="admin-section-block">
                    <h2>{ "Pending choreographies" }</h2>
                    <p class="admin-section-description">
                        { "Review submitted choreographies before they become available on DanceOmatic machines." }
                    </p>

                    { choreography_content }
                </section>
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
