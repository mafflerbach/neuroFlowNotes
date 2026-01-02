//! LM Studio embedding client.

use crate::types::{EmbeddingError, EmbeddingRequest, EmbeddingResponse};
use reqwest::Client;
use shared_types::EmbeddingSettings;
use std::time::Duration;
use tracing::{debug, warn};

/// Client for generating embeddings via LM Studio's OpenAI-compatible API.
#[derive(Clone)]
pub struct EmbeddingClient {
    client: Client,
    settings: EmbeddingSettings,
}

impl EmbeddingClient {
    /// Create a new embedding client with the given settings.
    pub fn new(settings: EmbeddingSettings) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, settings }
    }

    /// Get a reference to the current settings.
    pub fn settings(&self) -> &EmbeddingSettings {
        &self.settings
    }

    /// Update the client settings.
    pub fn update_settings(&mut self, settings: EmbeddingSettings) {
        self.settings = settings;
    }

    /// Check if the embedding service is reachable.
    pub async fn health_check(&self) -> Result<bool, EmbeddingError> {
        if !self.settings.enabled {
            return Ok(false);
        }

        let url = format!("{}/models", self.settings.endpoint_url);
        debug!("Checking embedding service health at: {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Embedding service is healthy");
                    Ok(true)
                } else {
                    warn!(
                        "Embedding service returned error status: {}",
                        response.status()
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("Embedding service health check failed: {}", e);
                Err(EmbeddingError::Unavailable(e.to_string()))
            }
        }
    }

    /// Generate an embedding for a single text.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InvalidResponse("Empty response".to_string()))
    }

    /// Generate embeddings for multiple texts in a single batch.
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        if !self.settings.enabled {
            return Err(EmbeddingError::Unavailable(
                "Embedding service is disabled".to_string(),
            ));
        }

        let url = format!("{}/embeddings", self.settings.endpoint_url);
        debug!("Generating embeddings for {} texts", texts.len());

        let request = EmbeddingRequest {
            model: self.settings.model.clone(),
            input: texts.to_vec(),
            encoding_format: "float".to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(EmbeddingError::Request)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::Api {
                message: format!("Status {}: {}", status, body),
            });
        }

        let embedding_response: EmbeddingResponse =
            response.json().await.map_err(EmbeddingError::Request)?;

        // Validate dimensions and sort by index
        let expected_dim = self.settings.dimensions as usize;
        let mut embeddings: Vec<(usize, Vec<f32>)> = embedding_response
            .data
            .into_iter()
            .map(|data| {
                if data.embedding.len() != expected_dim {
                    Err(EmbeddingError::DimensionMismatch {
                        expected: expected_dim,
                        actual: data.embedding.len(),
                    })
                } else {
                    Ok((data.index, data.embedding))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Sort by index to ensure correct order
        embeddings.sort_by_key(|(idx, _)| *idx);

        Ok(embeddings.into_iter().map(|(_, emb)| emb).collect())
    }

    /// Generate embeddings in batches, respecting the batch size setting.
    pub async fn embed_batched(
        &self,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let batch_size = self.settings.batch_size as usize;
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(batch_size) {
            let embeddings = self.embed_batch(chunk).await?;
            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_settings() -> EmbeddingSettings {
        EmbeddingSettings {
            enabled: true,
            endpoint_url: "http://localhost:1234/v1".to_string(),
            model: "test-model".to_string(),
            dimensions: 768,
            batch_size: 10,
        }
    }

    #[test]
    fn test_client_creation() {
        let settings = test_settings();
        let client = EmbeddingClient::new(settings.clone());
        assert_eq!(client.settings().model, "test-model");
    }

    #[test]
    fn test_settings_update() {
        let settings = test_settings();
        let mut client = EmbeddingClient::new(settings);

        let mut new_settings = test_settings();
        new_settings.model = "new-model".to_string();
        client.update_settings(new_settings);

        assert_eq!(client.settings().model, "new-model");
    }
}
