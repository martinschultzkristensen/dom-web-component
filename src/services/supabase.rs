use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;
use serde_json::Value;
use web_sys::File;

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
        "image.png".to_string()
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