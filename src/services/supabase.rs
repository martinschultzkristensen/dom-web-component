use gloo_net::http::Request;
use serde::Serialize;

const SUPABASE_URL: &str = "PASTE_SUPABASE_PROJECT_URL_HERE";
const SUPABASE_PUBLISHABLE_KEY: &str = "PASTE_SUPABASE_PUBLISHABLE_KEY_HERE";

#[derive(Serialize)]
pub struct NewDancer {
    pub name: String,
    pub strength: u8,
    pub flexibility: u8,
}

pub async fn insert_dancer(dancer: NewDancer) -> Result<(), String> {
    let url = format!("{}/rest/v1/dancers", SUPABASE_URL);

    let body = serde_json::to_string(&dancer)
        .map_err(|e| format!("Failed to serialize dancer: {}", e))?;

    let response = Request::post(&url)
        .header("apikey", SUPABASE_PUBLISHABLE_KEY)
        .header("Content-Type", "application/json")
        .header("Prefer", "return=minimal")
        .body(body)
        .map_err(|e| format!("Failed to build request: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {:?}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!(
            "Supabase insert failed: {} {}",
            response.status(),
            response.status_text()
        ))
    }
}