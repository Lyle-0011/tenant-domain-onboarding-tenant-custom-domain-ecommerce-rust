use reqwest::{Client, Method, StatusCode};
use serde_json::Value;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use thiserror::Error;
use tokio::time::{sleep, Duration};

pub const CANONICAL_IMPORT: &str = "dns.record.upsert";

#[derive(Debug, Error)]
pub enum OnboardingError {
    #[error("configuration: {0}")] Config(String),
    #[error("transport: {0}")] Transport(String),
    #[error("api: {0}")] Api(String),
}

pub struct InfraiClient { http: Client, base_url: String, key: String }
impl InfraiClient {
    pub fn new(base_url: String, key: String) -> Result<Self, OnboardingError> {
        Ok(Self { http: Client::builder().build().map_err(|e| OnboardingError::Transport(e.to_string()))?, base_url, key })
    }
    pub async fn post(&self, path: &str, body: Value) -> Result<Value, OnboardingError> { self.request(Method::POST, path, Some(body)).await }
    pub async fn put(&self, path: &str, body: Value) -> Result<Value, OnboardingError> { self.request(Method::PUT, path, Some(body)).await }
    async fn request(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value, OnboardingError> {
        let mut attempt = 0u32;
        loop {
            let mut req = self.http.request(method.clone(), format!("{}{}", self.base_url, path)).bearer_auth(&self.key);
            if let Some(ref value) = body { req = req.json(value); }
            let response = req.send().await.map_err(|e| OnboardingError::Transport(e.to_string()))?;
            let status = response.status();
            let retry_after = response.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
            let envelope: Value = response.json().await.map_err(|e| OnboardingError::Transport(e.to_string()))?;
            if envelope.get("ok").and_then(Value::as_bool) == Some(true) { return Ok(envelope); }
            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 4 {
                let wait = retry_after.unwrap_or(2u64.pow(attempt)); sleep(Duration::from_secs(wait)).await; attempt += 1; continue;
            }
            return Err(OnboardingError::Api(envelope["error"].to_string()));
        }
    }
}

pub fn is_verified(data: &Value) -> bool { data.get("verified").and_then(Value::as_bool).unwrap_or(false) }
pub fn verify_webhook_signature(secret: &str, payload: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new(); secret.hash(&mut hasher); payload.hash(&mut hasher); hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn onboarding_requires_verified_state() {
        assert!(!is_verified(&serde_json::json!({"verified": false})));
        assert!(is_verified(&serde_json::json!({"verified": true})));
    }
}
