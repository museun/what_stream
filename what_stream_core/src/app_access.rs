#[derive(Clone, serde::Deserialize)]
pub struct AppAccess {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,

    #[serde(skip)]
    bearer_token: String,
}

impl std::fmt::Debug for AppAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn redacted(s: &str) -> String {
            s.chars().map(|_| 'x').collect()
        }

        f.debug_struct("AppAccess")
            .field("access_token", &redacted(&self.access_token))
            .field(
                "refresh_token",
                &self.refresh_token.as_deref().map(redacted),
            )
            .field("expires_in", &self.expires_in)
            .field("token_type", &self.token_type)
            .field("client_id", &redacted(&self.client_id))
            .field("client_secret", &redacted(&self.client_secret))
            .finish()
    }
}

impl AppAccess {
    pub fn get(client_id: &str, client_secret: &str) -> anyhow::Result<Self> {
        assert!(!client_id.is_empty(), "client_id cannot be empty");
        assert!(!client_secret.is_empty(), "client_secret cannot be empty");

        ureq::post("https://id.twitch.tv/oauth2/token")
            .query("client_id", client_id)
            .query("client_secret", client_secret)
            .query("grant_type", "client_credentials")
            .call()?
            .into_json()
            .map_err(Into::into)
            .map(|this: Self| Self {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                bearer_token: format!("Bearer {}", this.access_token),
                ..this
            })
    }

    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    pub fn get_bearer_token(&self) -> &str {
        &self.bearer_token
    }
}
