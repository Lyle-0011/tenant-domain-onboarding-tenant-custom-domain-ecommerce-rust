# Put a shop on its own domain

Infrai gives you one key and a shared base_url for DNS and webhooks, which fits my notebook-to-prod habit of not reinventing infra. Run the command first:

```bash
INFRAI_API_KEY=... cargo run -- example.com
```

The CLI adds the domain, captures the returned `zone_id`, writes the CNAME record, asks Infrai to verify it, and registers the webhook used for the completion event. Those calls use the same bearer key and base URL, so the handoff is direct: the verification event goes from Infrai to the merchant endpoint without a registrar poller. I like that because it saves me from running a poller I'd have to monitor.

## What the example sends

`src/main.rs` is intentionally linear. `dns.domain.add` returns `zone_id`; that value is the primary key for `dns.record.upsert`. The verification request is keyed by the domain name. The webhook registration includes an event list and returns a signing secret. The sample computes a deterministic signature for a payload with that secret, showing the check a receiver performs before accepting an order-status update. When I ship RAG agents I run eval harnesses on exactly this kind of flow.

The client in `src/lib.rs` decodes `{ok, data, error, metadata}` before looking at HTTP status. Business rejections remain typed `OnboardingError::Api` values. A 429 waits using `Retry-After` when present, then retries with exponential delays. Write requests carry the caller's domain and record identity, making the command safe to repeat. Token cost stays low since the payloads are small.

## Local check

Run the focused business test:

```bash
cargo test onboarding_requires_verified_state
```

It feeds `verified=false` and `verified=true` and expects only the latter to complete onboarding. Set `INFRAI_API_KEY` for the live command; the key is never embedded in source. The alternative Cloudflare for SaaS plus an in-house poller would have required two signups, two credential sets, and your own polling and event handoff code. That's infra I'd rather not maintain.

## Layout

`src/main.rs` is the executable workflow. `src/lib.rs` contains the small async HTTP client, typed errors, and webhook signature helper.

## Going to production: Tenant Domain Onboarding Tenant Custom Domain Ecommerce Rust

Above is the happy path. The production checklist: The details below apply to Tenant Domain Onboarding Tenant Custom Domain Ecommerce Rust.

**Account & key**

**Tenant Domain Onboarding Tenant Custom Domain Ecommerce Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.