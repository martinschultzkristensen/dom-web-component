use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::{window, File};

const SUPABASE_URL: &str = "https://tfrkkrbfgdgsbwqcrcqq.supabase.co";
const SUPABASE_PUBLISHABLE_KEY: &str = "sb_publishable_EKhP5Y6SwcqoIEAdeob62w_xOhFZI-s";

const ACCESS_TOKEN_KEY: &str = "danceomatic_access_token";
const REFRESH_TOKEN_KEY: &str = "danceomatic_refresh_token";
const USER_ID_KEY: &str = "danceomatic_user_id";
const USER_EMAIL_KEY: &str = "danceomatic_user_email";

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthUser {
    pub id: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user: AuthUser,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub email: Option<String>,
    pub role: String,
}

fn local_storage() -> Result<web_sys::Storage, String> {
    window()
        .ok_or("No browser window found".to_string())?
        .local_storage()
        .map_err(|e| format!("Could not access localStorage: {:?}", e))?
        .ok_or("localStorage is not available".to_string())
}

pub fn save_session(session: &AuthSession) -> Result<(), String> {
    let storage = local_storage()?;

    storage
        .set_item(ACCESS_TOKEN_KEY, &session.access_token)
        .map_err(|e| format!("Could not save access token: {:?}", e))?;

    storage
        .set_item(REFRESH_TOKEN_KEY, &session.refresh_token)
        .map_err(|e| format!("Could not save refresh token: {:?}", e))?;

    storage
        .set_item(USER_ID_KEY, &session.user.id)
        .map_err(|e| format!("Could not save user id: {:?}", e))?;

    if let Some(email) = &session.user.email {
        storage
            .set_item(USER_EMAIL_KEY, email)
            .map_err(|e| format!("Could not save user email: {:?}", e))?;
    }

    Ok(())
}

pub fn logout() -> Result<(), String> {
    let storage = local_storage()?;

    storage
        .remove_item(ACCESS_TOKEN_KEY)
        .map_err(|e| format!("Could not remove access token: {:?}", e))?;

    storage
        .remove_item(REFRESH_TOKEN_KEY)
        .map_err(|e| format!("Could not remove refresh token: {:?}", e))?;

    storage
        .remove_item(USER_ID_KEY)
        .map_err(|e| format!("Could not remove user id: {:?}", e))?;

    storage
        .remove_item(USER_EMAIL_KEY)
        .map_err(|e| format!("Could not remove user email: {:?}", e))?;

    Ok(())
}

pub fn get_access_token() -> Option<String> {
    local_storage()
        .ok()
        .and_then(|storage| storage.get_item(ACCESS_TOKEN_KEY).ok().flatten())
}

pub fn get_current_user_id() -> Option<String> {
    local_storage()
        .ok()
        .and_then(|storage| storage.get_item(USER_ID_KEY).ok().flatten())
}

pub fn is_logged_in() -> bool {
    get_access_token().is_some()
}

pub async fn login(email: String, password: String) -> Result<AuthSession, String> {
    let url = format!("{}/auth/v1/token?grant_type=password", SUPABASE_URL);

    let body = LoginRequest { email, password };

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Could not build login request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Login request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown login error".to_string());

        return Err(format!("Login failed: {} {}", status, error_text));
    }

    let session: AuthSession = response
        .json()
        .await
        .map_err(|e| format!("Could not read login response: {:?}", e))?;

    save_session(&session)?;

    Ok(session)
}

pub async fn signup(email: String, password: String) -> Result<Option<AuthSession>, String> {
    let url = format!("{}/auth/v1/signup", SUPABASE_URL);

    let body = LoginRequest { email, password };

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Could not build signup request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Signup request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown signup error".to_string());

        return Err(format!("Signup failed: {} {}", status, error_text));
    }

    let value: Value = response
        .json()
        .await
        .map_err(|e| format!("Could not read signup response: {:?}", e))?;

    let access_token = value
        .get("access_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let refresh_token = value
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let user_value = value.get("user").cloned();

    if let (Some(access_token), Some(refresh_token), Some(user_value)) =
        (access_token, refresh_token, user_value)
    {
        let user: AuthUser = serde_json::from_value(user_value)
            .map_err(|e| format!("Could not read signup user: {:?}", e))?;

        let session = AuthSession {
            access_token,
            refresh_token,
            user,
        };

        save_session(&session)?;

        Ok(Some(session))
    } else {
        Ok(None)
    }
}

pub async fn get_my_profile() -> Result<Profile, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/rest/v1/profiles?select=id,email,role&id=eq.{}",
        SUPABASE_URL,
        get_current_user_id().ok_or("Missing user id".to_string())?
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Profile request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown profile error".to_string());

        return Err(format!("Profile request failed: {} {}", status, error_text));
    }

    let mut profiles: Vec<Profile> = response
        .json()
        .await
        .map_err(|e| format!("Could not read profile response: {:?}", e))?;

    profiles
        .pop()
        .ok_or("Profile not found for current user".to_string())
}

#[derive(Serialize)]
pub struct NewDancer {
    pub created_by: String,
    pub name: String,
    pub image_path: Option<String>,
    pub strength: u8,
    pub flexibility: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DancerRow {
    pub id: String,
    pub name: String,
    pub image_path: Option<String>,
    pub strength: u8,
    pub flexibility: u8,
    pub status: String,
    pub visibility: String,
}

pub async fn insert_dancer(dancer: NewDancer) -> Result<(), String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!("{}/rest/v1/dancers", SUPABASE_URL);

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&dancer)
        .map_err(|e| format!("Could not build dancer request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Dancer insert failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown dancer insert error".to_string());

        Err(format!("Dancer insert failed: {} {}", status, error_text))
    }
}

#[derive(Serialize)]
struct UpdateDancerRequest {
    name: String,
    image_path: Option<String>,
    strength: u8,
    flexibility: u8,
}

pub async fn update_dancer(
    dancer_id: String,
    name: String,
    image_path: Option<String>,
    strength: u8,
    flexibility: u8,
) -> Result<(), String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!("{}/rest/v1/dancers?id=eq.{}", SUPABASE_URL, dancer_id);

    let body = UpdateDancerRequest {
        name,
        image_path,
        strength,
        flexibility,
    };

    let response = Request::patch(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .map_err(|e| format!("Could not build dancer update request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Dancer update failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown dancer update error".to_string());

        Err(format!("Dancer update failed: {} {}", status, error_text))
    }
}
#[derive(Debug, Deserialize)]
struct DancerUsageChoreography {
    title: String,
}

#[derive(Debug, Deserialize)]
struct DancerUsageRow {
    choreographies: Option<DancerUsageChoreography>,
}

async fn fetch_dancer_choreography_titles(
    dancer_id: &str,
) -> Result<Vec<String>, String> {
    let access_token =
        get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/rest/v1/choreography_dancers?select=choreographies(title)&dancer_id=eq.{}",
        SUPABASE_URL,
        dancer_id
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|_| "Could not check whether the dancer is in use.".to_string())?;

    if !response.ok() {
        return Err("Could not check whether the dancer is in use.".to_string());
    }

    let rows: Vec<DancerUsageRow> = response
        .json()
        .await
        .map_err(|_| "Could not check whether the dancer is in use.".to_string())?;

    let mut titles = Vec::<String>::new();

    for row in rows {
        if let Some(choreography) = row.choreographies {
            if !titles.contains(&choreography.title) {
                titles.push(choreography.title);
            }
        }
    }

    Ok(titles)
}

pub async fn delete_dancer(dancer_id: String) -> Result<(), String> {
    let choreography_titles =
        fetch_dancer_choreography_titles(&dancer_id).await?;

    if !choreography_titles.is_empty() {
        let message = if choreography_titles.len() == 1 {
            format!(
                "Cannot remove dancer because it is used in the choreography \"{}\".",
                choreography_titles[0]
            )
        } else {
            let names = choreography_titles
                .iter()
                .map(|title| format!("\"{}\"", title))
                .collect::<Vec<String>>()
                .join(", ");

            format!(
                "Cannot remove dancer because it is used in these choreographies: {}.",
                names
            )
        };

        return Err(message);
    }

    let access_token =
        get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/rest/v1/dancers?id=eq.{}",
        SUPABASE_URL,
        dancer_id
    );

    let response = Request::delete(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Prefer", "return=minimal")
        .send()
        .await
        .map_err(|_| "Could not remove dancer.".to_string())?;

    if response.ok() {
        Ok(())
    } else {
        Err("Could not remove dancer.".to_string())
    }
}

pub async fn fetch_dancers() -> Result<Vec<DancerRow>, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/rest/v1/dancers?select=id,name,image_path,strength,flexibility,status,visibility&order=created_at.desc",
        SUPABASE_URL
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Dancers request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown dancers fetch error".to_string());

        return Err(format!("Dancers fetch failed: {} {}", status, error_text));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Could not read dancers response: {:?}", e))
}

#[derive(Serialize)]
pub struct NewChoreography {
    pub created_by: String,
    pub title: String,
    pub duration_seconds: i32,
    pub description: String,
    pub image_path: String,
    pub demo_video_path: String,
    pub choreo_video_path: String,
}

#[derive(Debug, Deserialize)]
struct InsertedChoreographyRow {
    id: String,
}

#[derive(Serialize)]
struct NewChoreographyDancer {
    choreography_id: String,
    dancer_id: String,
    sort_order: i32,
}

async fn insert_choreography(choreography: NewChoreography) -> Result<String, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!("{}/rest/v1/choreographies?select=id", SUPABASE_URL);

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&choreography)
        .map_err(|e| format!("Could not build choreography insert request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Choreography insert failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown choreography insert error".to_string());

        return Err(format!(
            "Choreography insert failed: {} {}",
            status, error_text
        ));
    }

    let mut rows: Vec<InsertedChoreographyRow> = response
        .json()
        .await
        .map_err(|e| format!("Could not read choreography insert response: {:?}", e))?;

    rows.pop()
        .map(|row| row.id)
        .ok_or("Choreography insert returned no id".to_string())
}

async fn insert_choreography_dancers(
    choreography_id: &str,
    dancer_ids: Vec<String>,
) -> Result<(), String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    if dancer_ids.is_empty() {
        return Ok(());
    }

    let rows = dancer_ids
        .into_iter()
        .enumerate()
        .map(|(index, dancer_id)| NewChoreographyDancer {
            choreography_id: choreography_id.to_string(),
            dancer_id,
            sort_order: index as i32 + 1,
        })
        .collect::<Vec<NewChoreographyDancer>>();

    let url = format!("{}/rest/v1/choreography_dancers", SUPABASE_URL);

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&rows)
        .map_err(|e| format!("Could not build choreography dancers request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Choreography dancers insert failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown choreography dancers insert error".to_string());

        Err(format!(
            "Choreography dancers insert failed: {} {}",
            status, error_text
        ))
    }
}

pub async fn submit_choreography(
    title: String,
    duration_seconds: i32,
    description: String,
    image_path: String,
    demo_video_path: String,
    choreo_video_path: String,
    dancer_ids: Vec<String>,
) -> Result<String, String> {
    let created_by = get_current_user_id().ok_or("Missing user id".to_string())?;

    let choreography_id = insert_choreography(NewChoreography {
        created_by,
        title,
        duration_seconds,
        description,
        image_path,
        demo_video_path,
        choreo_video_path,
    })
    .await?;

    insert_choreography_dancers(&choreography_id, dancer_ids).await?;

    Ok(choreography_id)
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubmittedChoreographyRow {
    pub id: String,
    pub title: String,
    pub duration_seconds: i32,
    pub image_path: String,
    pub status: String,
    pub created_at: String,
    pub submitted_at: Option<String>,
}

pub async fn fetch_submitted_choreographies() -> Result<Vec<SubmittedChoreographyRow>, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;
    let user_id = get_current_user_id().ok_or("Missing user id".to_string())?;

    let url = format!(
        "{}/rest/v1/choreographies?select=id,title,duration_seconds,image_path,status,created_at,submitted_at&created_by=eq.{}&order=created_at.desc",
        SUPABASE_URL,
        user_id
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Submitted choreographies request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown submitted choreographies fetch error".to_string());

        return Err(format!(
            "Submitted choreographies fetch failed: {} {}",
            status, error_text
        ));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Could not read submitted choreographies response: {:?}", e))
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminDancerInfoRow {
    pub id: String,
    pub name: String,
    pub image_path: Option<String>,
    pub strength: u8,
    pub flexibility: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminChoreographyDancerRow {
    pub dancer_id: String,
    pub sort_order: i32,
    pub dancers: Option<AdminDancerInfoRow>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminChoreographyRow {
    pub id: String,
    pub title: String,
    pub duration_seconds: i32,
    pub description: String,
    pub image_path: String,
    pub demo_video_path: String,
    pub choreo_video_path: String,
    pub status: String,
    pub created_at: String,
    pub submitted_at: Option<String>,

    #[serde(default)]
    pub choreography_dancers: Vec<AdminChoreographyDancerRow>,
}

pub async fn fetch_admin_pending_choreographies() -> Result<Vec<AdminChoreographyRow>, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/rest/v1/choreographies?select=id,title,duration_seconds,description,image_path,demo_video_path,choreo_video_path,status,created_at,submitted_at,choreography_dancers(dancer_id,sort_order,dancers(id,name,image_path,strength,flexibility))&status=eq.pending&order=created_at.asc",
        SUPABASE_URL
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Admin pending choreographies request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown admin pending choreographies error".to_string());

        return Err(format!(
            "Admin pending choreographies fetch failed: {} {}",
            status, error_text
        ));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Could not read admin pending choreographies response: {:?}", e))
}

#[derive(Serialize)]
struct UpdateChoreographyStatusRequest {
    status: String,
}

pub async fn update_choreography_status(
    choreography_id: String,
    status: String,
) -> Result<(), String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let normalized_status = status.trim().to_lowercase();

    if normalized_status != "pending"
        && normalized_status != "approved"
        && normalized_status != "rejected"
    {
        return Err("Invalid choreography status".to_string());
    }

    let url = format!(
        "{}/rest/v1/choreographies?id=eq.{}",
        SUPABASE_URL,
        choreography_id
    );

    let body = UpdateChoreographyStatusRequest {
        status: normalized_status,
    };

    let response = Request::patch(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .map_err(|e| format!("Could not build choreography status update request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Choreography status update failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown choreography status update error".to_string());

        Err(format!(
            "Choreography status update failed: {} {}",
            status, error_text
        ))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct MachineMediaRow {
    pub id: String,
    pub machine_id: String,
    pub intro_video_path: Option<String>,
    pub load_video_path: Option<String>,
    pub updated_at: String,
}

fn is_valid_machine_id(machine_id: &str) -> bool {
    matches!(machine_id, "machine_1" | "machine_2" | "machine_3")
}

pub async fn fetch_machine_media() -> Result<Vec<MachineMediaRow>, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/rest/v1/machine_media?select=id,machine_id,intro_video_path,load_video_path,updated_at&order=machine_id.asc",
        SUPABASE_URL
    );

    let response = Request::get(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Machine media request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown machine media fetch error".to_string());

        return Err(format!("Machine media fetch failed: {} {}", status, error_text));
    }

    response
        .json()
        .await
        .map_err(|e| format!("Could not read machine media response: {:?}", e))
}

pub async fn upload_machine_media_video(
    file: File,
    machine_id: &str,
    media_role: &str,
) -> Result<String, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    if !is_valid_machine_id(machine_id) {
        return Err("Invalid machine id".to_string());
    }

    if media_role != "intro_video" && media_role != "load_video" {
        return Err("Invalid machine media role".to_string());
    }

    let file_name = sanitize_file_name(&file.name());
    let timestamp = js_sys::Date::now() as u64;

    let file_path = format!(
        "machine-media/{}/{}_{}_{}",
        machine_id,
        media_role,
        timestamp,
        file_name
    );

    let content_type = if file.type_().is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.type_()
    };

    let url = format!(
        "{}/storage/v1/object/choreography-files/{}",
        SUPABASE_URL,
        file_path
    );

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", &content_type)
        .header("x-upsert", "true")
        .body(file)
        .map_err(|e| format!("Could not build machine media upload request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Machine media upload failed: {:?}", e))?;

    if response.ok() {
        Ok(file_path)
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown machine media upload error".to_string());

        Err(format!(
            "Machine media upload failed: {} {}",
            status,
            error_text
        ))
    }
}

pub async fn update_machine_media(
    machine_id: String,
    intro_video_path: Option<String>,
    load_video_path: Option<String>,
) -> Result<(), String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;
    let user_id = get_current_user_id().ok_or("Missing user id".to_string())?;

    if !is_valid_machine_id(&machine_id) {
        return Err("Invalid machine id".to_string());
    }

    let updated_at = js_sys::Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_string());

    let body = serde_json::json!({
        "intro_video_path": intro_video_path,
        "load_video_path": load_video_path,
        "updated_by": user_id,
        "updated_at": updated_at,
    });

    let url = format!(
        "{}/rest/v1/machine_media?machine_id=eq.{}",
        SUPABASE_URL,
        machine_id
    );

    let response = Request::patch(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .json(&body)
        .map_err(|e| format!("Could not build machine media update request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Machine media update failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown machine media update error".to_string());

        Err(format!(
            "Machine media update failed: {} {}",
            status, error_text
        ))
    }
}

fn sanitize_file_name(file_name: &str) -> String {
    let cleaned: String = file_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if cleaned.trim().is_empty() {
        "file".to_string()
    } else {
        cleaned
    }
}

pub async fn upload_dancer_image(file: File) -> Result<String, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;
    let user_id = get_current_user_id().ok_or("Missing user id".to_string())?;

    let file_name = sanitize_file_name(&file.name());
    let timestamp = js_sys::Date::now() as u64;

    let file_path = format!("{}/dancers/{}_{}", user_id, timestamp, file_name);

    let content_type = if file.type_().is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.type_()
    };

    let url = format!(
        "{}/storage/v1/object/dancer-images/{}",
        SUPABASE_URL,
        file_path
    );

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", &content_type)
        .header("x-upsert", "true")
        .body(file)
        .map_err(|e| format!("Could not build image upload request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Image upload failed: {:?}", e))?;

    if response.ok() {
        Ok(file_path)
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown image upload error".to_string());

        Err(format!("Image upload failed: {} {}", status, error_text))
    }
}

pub async fn upload_choreography_file(
    file: File,
    file_role: &str,
) -> Result<String, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;
    let user_id = get_current_user_id().ok_or("Missing user id".to_string())?;

    let file_name = sanitize_file_name(&file.name());
    let timestamp = js_sys::Date::now() as u64;

    let file_path = format!(
        "{}/choreography-drafts/{}_{}_{}",
        user_id,
        file_role,
        timestamp,
        file_name
    );

    let content_type = if file.type_().is_empty() {
        "application/octet-stream".to_string()
    } else {
        file.type_()
    };

    let url = format!(
        "{}/storage/v1/object/choreography-files/{}",
        SUPABASE_URL,
        file_path
    );

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", &content_type)
        .header("x-upsert", "true")
        .body(file)
        .map_err(|e| format!("Could not build choreography file upload request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Choreography file upload failed: {:?}", e))?;

    if response.ok() {
        Ok(file_path)
    } else {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown choreography file upload error".to_string());

        Err(format!(
            "Choreography file upload failed: {} {}",
            status,
            error_text
        ))
    }
}

#[derive(Serialize)]
struct SignedUrlRequest {
    #[serde(rename = "expiresIn")]
    expires_in: u32,
}

#[derive(Debug, Deserialize)]
struct SignedUrlResponse {
    #[serde(rename = "signedURL")]
    signed_url: String,
}

pub async fn create_signed_url(path: &str) -> Result<String, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/storage/v1/object/sign/dancer-images/{}",
        SUPABASE_URL,
        path
    );

    let body = SignedUrlRequest {
        expires_in: 60 * 60,
    };

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Could not build signed URL request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Signed URL request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown signed URL error".to_string());

        return Err(format!("Signed URL failed: {} {}", status, error_text));
    }

    let signed_response: SignedUrlResponse = response
        .json()
        .await
        .map_err(|e| format!("Could not read signed URL response: {:?}", e))?;

    if signed_response.signed_url.starts_with("http") {
        Ok(signed_response.signed_url)
    } else {
        Ok(format!(
            "{}/storage/v1{}",
            SUPABASE_URL,
            signed_response.signed_url
        ))
    }
}

pub async fn create_choreography_file_signed_url(path: &str) -> Result<String, String> {
    let access_token = get_access_token().ok_or("User is not logged in".to_string())?;

    let url = format!(
        "{}/storage/v1/object/sign/choreography-files/{}",
        SUPABASE_URL,
        path
    );

    let body = SignedUrlRequest {
        expires_in: 60 * 60,
    };

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .map_err(|e| format!("Could not build choreography signed URL request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Choreography signed URL request failed: {:?}", e))?;

    if !response.ok() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown choreography signed URL error".to_string());

        return Err(format!("Choreography signed URL failed: {} {}", status, error_text));
    }

    let signed_response: SignedUrlResponse = response
        .json()
        .await
        .map_err(|e| format!("Could not read choreography signed URL response: {:?}", e))?;

    if signed_response.signed_url.starts_with("http") {
        Ok(signed_response.signed_url)
    } else {
        Ok(format!(
            "{}/storage/v1{}",
            SUPABASE_URL,
            signed_response.signed_url
        ))
    }
}