# Webhook Classifier — AID Language Showcase

A complete, production-ready webhook classification application built entirely in AID. This is **the** showcase demo that proves AID works for real-world use cases.

## What It Does

Receives webhooks from any source (GitHub, Stripe, PagerDuty, etc.), automatically classifies them using AI `reason` blocks, tracks accuracy with `evolve` blocks, and displays everything on a live dashboard.

## AID Features Used

| Feature | Usage |
|---------|-------|
| `std.db` | SQLite storage for webhooks + classifications + audit log |
| `std.env` | Configuration via `.env` (port, DB path, API keys) |
| `std.auth` | JWT-protected admin endpoints, API key for webhook ingestion |
| `std.html` | Dashboard page with live stats and webhook table |
| `reason` blocks | AI classification: `classify_webhook`, `detect_priority`, `extract_source` |
| `evolve` blocks | Track classification accuracy, trigger retraining at thresholds |
| `contract` blocks | Validate webhook payload structure with natural language rules |
| `intent` routing | Auto-discovered CRUD routes from handler naming convention |

## Architecture

```
POST /api/webhooks  →  contract validation  →  reason classification  →  SQLite storage
                                                        ↓
GET /dashboard      ←  html.template        ←  db.query stats
GET /api/stats      ←  JSON response        ←  db.query aggregation
GET /telemetry      ←  evolve telemetry     ←  .cortex/telemetry/
```

## Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/` | Public | Service info + endpoint list |
| GET | `/health` | Public | Health check |
| POST | `/auth/login` | Public | Get JWT token |
| POST | `/api/webhooks` | API Key | Ingest + classify a webhook |
| GET | `/api/webhooks` | JWT | List recent webhooks |
| GET | `/api/stats` | JWT | Classification statistics |
| GET | `/dashboard` | JWT | HTML dashboard |
| POST | `/api/validate` | Public | Validate webhook payload |
| GET | `/telemetry` | Public | Evolve block telemetry |

## Quick Start

```bash
# 1. Build the AID compiler
cd ~/Documents/projects/aid-lang/compiler
source ~/.cargo/env
cargo build --release

# 2. Setup the webhook classifier
cd ../examples/webhook-classifier
cp .env.example .env

# 3. Build and run
../../compiler/target/release/aid build main.aid
./../../build/aid-webhook_classifier

# 4. Test it
curl http://localhost:8080/
curl http://localhost:8080/health

# Ingest a webhook
curl -X POST http://localhost:8080/api/webhooks \
  -H "x-api-key: whk_live_your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{"source":"github","event":"push","message":"Production server is down"}'

# Get JWT token
curl -X POST http://localhost:8080/auth/login

# View dashboard (in browser)
open http://localhost:8080/dashboard
```

## Classification Categories

The `classify_webhook` reason block classifies into:

| Category | Trigger Keywords |
|----------|-----------------|
| **urgent** | outage, down, critical, incident |
| **billing** | payment, invoice, charge, subscription |
| **technical** | error, bug, API, endpoint |
| **security** | vulnerability, breach, unauthorized, exploit |
| **deployment** | deploy, release, rollback, build |
| **monitoring** | CPU, memory, alert, threshold |
| **notification** | registration, message, digest |
| **spam** | free offer, promotion, click here |

## File Structure

```
webhook-classifier/
├── main.aid              # Full application (all AID features)
├── .env.example          # Configuration template
├── README.md             # This file
├── migrations/
│   ├── 001_create_webhooks.sql   # Schema: tables + indexes
│   └── 002_seed_data.sql         # Seed: 12 real webhook examples
├── templates/
│   └── dashboard.html    # Dashboard HTML template
└── public/
    └── style.css         # Dark theme dashboard CSS
```

## License

Part of the AID Language project. BSL 1.1.
