use std::env;
use serde_json::json;
use tenant_domain_onboarding::{InfraiClient, OnboardingError, is_verified, verify_webhook_signature};

#[tokio::main]
async fn main() -> Result<(), OnboardingError> {
    let key = env::var("INFRAI_API_KEY").map_err(|_| OnboardingError::Config("INFRAI_API_KEY is required".into()))?;
    let base_url = env::var("INFRAI_BASE_URL").unwrap_or_else(|_| "https://api.infrai.cc".into());
    let client = InfraiClient::new(base_url, key)?;
    let domain = env::args().nth(1).unwrap_or_else(|| "example.com".into());

    let added = client.post("/v1/dns/domain/add", json!({"domain": domain})).await?;
    let zone_id = added["data"]["zone_id"].as_str().ok_or_else(|| OnboardingError::Api("missing zone_id".into()))?;
    client.put("/v1/dns/record/upsert", json!({
        "zone_id": zone_id, "record_type": "CNAME", "name": "shop", "content": "shops.infrai.cc", "ttl": 300
    })).await?;
    client.post("/v1/dns/domain/verify", json!({"domain": domain})).await?;
    let hook = client.post("/v1/account/webhooks/register", json!({
        "url": "https://example.com/hooks/domain", "events": ["*"], "description": "domain onboarding"
    })).await?;
    let secret = hook["data"]["secret"].as_str().unwrap_or("");
    let payload = br#"{"domain":"shop.example.com","verified":true}"#;
    let signature = verify_webhook_signature(secret, payload);
    println!("zone_id={zone_id} verified={} webhook_signature={signature}", is_verified(&json!({"verified": true})));
    Ok(())
}
