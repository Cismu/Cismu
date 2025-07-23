use anyhow::Result;
use serde::Deserialize;

// Structs para parsear la respuesta JSON de la API
#[derive(Debug, Deserialize)]
pub struct AcoustidResult {
    pub id: String, // El AcoustID único (UUID)
    pub score: f32,
    #[serde(default)]
    pub recordings: Vec<Recording>,
}

#[derive(Debug, Deserialize)]
pub struct Recording {
    pub id: String, // El ID de la grabación en MusicBrainz (UUID)
}

#[derive(Debug, Deserialize)]
struct AcoustidResponse {
    status: String,
    results: Vec<AcoustidResult>,
}

// El cliente que hará las peticiones
#[derive(Debug, Clone)]
pub struct AcoustidClient {
    api_key: String,
    client: reqwest::Client,
}

impl AcoustidClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Busca un fingerprint en la API de AcoustID.
    pub async fn lookup(&self, fingerprint: &str, duration_secs: u32) -> Result<Vec<AcoustidResult>> {
        let url = "https://api.acoustid.org/v2/lookup";

        let response = self
            .client
            .post(url)
            .form(&[
                ("client", self.api_key.as_str()),
                // Pedimos los IDs de MusicBrainz, son muy valiosos
                ("meta", "recordings"),
                ("duration", &duration_secs.to_string()),
                ("fingerprint", fingerprint),
            ])
            .send()
            .await?;

        let response = response.json::<AcoustidResponse>().await?;

        Ok(response.results)
    }
}
