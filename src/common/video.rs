use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct CreateRoomRequest {
    privacy: String,
}

#[derive(Deserialize)]
struct CreateRoomResponse {
    friendly_url: String,
}

#[derive(Clone)]
pub struct VideoClient {
    http: reqwest::Client,
    api_key: String,
    // Rooms are hosted under YOUR chosen subdomain, e.g.
    // https://{team_name}.digitalsamba.com/{friendly_url} — this is
    // set when you sign up and is separate from the API key itself.
    team_name: String,
}

impl VideoClient {
    pub fn from_env() -> Self {
        let api_key =
            std::env::var("DIGITAL_SAMBA_API_KEY").expect("DIGITAL_SAMBA_API_KEY must be set");
        let team_name = std::env::var("DIGITAL_SAMBA_TEAM_NAME")
            .expect("DIGITAL_SAMBA_TEAM_NAME must be set");

        VideoClient {
            http: reqwest::Client::new(),
            api_key,
            team_name,
        }
    }

    // Creates a public room and returns its full joinable URL. We let
    // Digital Samba auto-generate the room's unique slug (`friendly_url`)
    // rather than inventing our own — one less thing to get wrong.
    pub async fn create_room(&self) -> Result<String, String> {
        let response = self
            .http
            .post("https://api.digitalsamba.com/api/v1/rooms")
            .bearer_auth(&self.api_key)
            .json(&CreateRoomRequest {
                privacy: "public".to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!(
                "Digital Samba returned status {}",
                response.status()
            ));
        }

        let parsed: CreateRoomResponse = response.json().await.map_err(|e| e.to_string())?;

        Ok(format!(
            "https://{}.digitalsamba.com/{}",
            self.team_name, parsed.friendly_url
        ))
    }
}