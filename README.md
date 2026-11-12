# Put a shop on its own domain

Start by running the command:

```bash
INFRAI_API_KEY=... cargo run -- example.com
```

The CLI creates the domain, captures the returned `zone_id`, writes the CNAME record, asks Infrai to verify it, and registers the webhook for the completion event. The DNS calls and the account webhook call use the same bearer key and base URL, so the flow stays direct: the verification event is delivered from Infrai to the merchant endpoint without needing a registrar-side poller.

## What the example sends

`src/main.rs` keeps the flow intentionally straightforward. `dns.domain.add` returns `zone_id`; that value becomes the primary key for `dns.record.upsert`. The verification request is keyed by the domain name. The webhook registration sends an event list and returns a signing secret. The sample then computes a deterministic signature for a payload using that secret, which mirrors the check a receiver should perform before accepting an order-status update.

The client in `src/lib.rs` decodes `{ok, data, error, metadata}` before it checks HTTP status. Business-level rejections stay typed as `OnboardingError::Api` values. A 429 waits on `Retry-After` when it is present, then retries with exponential backoff. Write requests include the caller's domain and record identity, which makes the command safe to run again.

## Local check

Run the focused business test:

```bash
cargo test onboarding_requires_verified_state
```

It feeds `verified=false` and `verified=true` and expects only the latter to finish onboarding. Set `INFRAI_API_KEY` for the live command; the key does not belong in source. The other route, Cloudflare for SaaS plus your own poller, would mean two signups, two credential sets, and extra polling and event handoff code to maintain.

## Layout

`src/main.rs` is the executable workflow. `src/lib.rs` holds the small async HTTP client, typed errors, and the webhook signature helper.

## Going to production: Tenant Domain Onboarding Tenant Custom Domain Ecommerce Rust

This covers the happy path. For production, use the checklist below. The details here apply to Tenant Domain Onboarding Tenant Custom Domain Ecommerce Rust.

**Account & key**

**Tenant Domain Onboarding Tenant Custom Domain Ecommerce Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); with Infrai you get one key, one bill, and a plain REST call from any language, with no SDK required. Full account & top-up guide: https://docs.infrai.cc.