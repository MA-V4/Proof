# PROOF

**Financial logic, verified pure.**

[![Build](https://github.com/MA-V4/Proof/actions/workflows/ci.yml/badge.svg)](https://github.com/MA-V4/Proof/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org)

PROOF is an open-source financial logic verification platform. It makes it impossible to silently ship wrong financial logic.

---

## The problem

Every financial product has three versions of its rules.

**Version 1.** The product spec. A Word document. What the product manager wrote. What the FCA-approved terms sheet describes.

**Version 2.** The human interpretation. What the compliance officer thinks it means. What the engineer understood from a ticket.

**Version 3.** The actual running code.

These three versions are never formally verified to match each other. The gap between them is where billions of pounds disappear every year.

TSB: £330M. Santander: £130M. NatWest: £200M. Barclays: multiple FCA enforcement actions. Every single one was a divergence between what the system was supposed to do and what it actually did, invisible until it was catastrophically too late.

PROOF closes that gap permanently.

---

## How it works

Your .proof spec Your running system
| |
v v
Evaluator System output
| |
+---------> Comparator <-------+
|
Divergence detected?
/
No Yes
| |
OK Alert + log entry
|
Dashboard update
|
FCA audit trail


PROOF runs alongside your existing system in production. Every financial event your system processes is independently recomputed by the PROOF evaluation engine. When outputs match, nothing happens. When they diverge, PROOF tells you immediately, before the customer is affected, with an exact field-level diff.

---

## Quick start

**Prerequisites:** Rust 1.75+, Node.js 20+

```bash
git clone https://github.com/MA-V4/Proof
cd Proof

# Build everything
cargo build

# Start the server (loads .proof files from examples/)
cargo run --bin proof-server

# In a second terminal, start the dashboard
cd dashboard && npm install && npm run dev
```

Open `http://localhost:3000`. The dashboard connects to the API at `http://localhost:3001`.

---

## The specification language

PROOF introduces a domain-specific language for expressing financial product rules as executable specifications. A savings account:

product SavingsAccount {
jurisdiction: UK
regulator: FCA
category: deposit

interest {
base_rate: 4.50%

tiers {
  when balance >= 10_000  rate: base_rate + 1.00%
  when balance >= 1_000   rate: base_rate + 0.50%
  otherwise               rate: base_rate
}

promotional {
  condition:      days_since_joined <= 90
  rate:           base_rate + 2.00%
  expires_after:  90 days
  non_renewable:  true
}

accrual {
  frequency:        daily
  basis:            ACT/365
  compound:         annually
  minimum_payable:  GBP 0.01
}

}

protection {
scheme: FSCS
limit: GBP 85_000
}

obligations {
cooling_off: 14 days
rate_change_notice: 14 days
annual_summary: required
}
}


This is not configuration. It is an executable specification. PROOF runs it independently of your implementation and compares outputs at every calculation point.

---

## CLI reference

**Evaluate a spec:**
```bash
proof check examples/savings-account.proof \
  --input '{"balance": 9840, "event": "daily_accrual"}'
```

PROOF v0.1.0

Specification: SavingsAccount
Event: daily_accrual
Balance: £9840

Tier: tier_2
Rate: 5.00%
Result: £1.35

Spec check: OK


**Verify a batch of system events:**
```bash
proof verify examples/savings-account.proof --batch examples/events.ndjson
```

PROOF v0.1.0
Spec: SavingsAccount File: examples/events.ndjson

DIVERGENCE
Customer: C-001 Balance: £9840
Event: daily_accrual

Field Spec System Delta
amount 1.35 1.21 -0.14
rate_applied 5.00 4.50 -0.50
applied_tier tier_2 base

OK C-003 £500

Verified: 3 Divergences: 1


**Simulate a spec change:**
```bash
proof simulate examples/savings-account.proof \
  --new examples/savings-account-v2.proof \
  --portfolio examples/portfolio.ndjson
```

Customers worse off: 4
avg daily impact: £-0.0225/day
Customers better off: 0
Customers neutral: 6
Monthly aggregate: £-2.70

Regulatory flags
FCA Consumer Duty
4 customers receive a material rate reduction
Action: 14-day advance notice required. Individual notification mandatory.
Notice: 14 days

Verdict: DEPLOY WITH REVIEW


**Diff two spec versions:**
```bash
proof diff examples/savings-account.proof examples/savings-account-v2.proof
```

PROOF diff

1 change:

tier_2 threshold
£1000 -> £2500


**Dump the parsed AST:**
```bash
proof ir examples/savings-account.proof
```

---

## Event ingestion format

Send events to the server via HTTP, or verify in bulk using batch NDJSON files.

**Single event (HTTP):**
```json
{
  "customer_id": "C-001",
  "event_type": { "type": "daily_accrual" },
  "balance": "9840",
  "system_output": {
    "amount": "1.21",
    "applied_tier": "base",
    "rate_applied": "4.50"
  }
}
```

**Batch file (NDJSON - one event per line):**
```json
{"customer_id":"C-001","event_type":{"type":"daily_accrual"},"balance":"9840","system_output":{"amount":"1.21","applied_tier":"base","rate_applied":"4.50"}}
{"customer_id":"C-002","event_type":{"type":"daily_accrual"},"balance":"12000","system_output":{"amount":"1.97","applied_tier":"tier_1","rate_applied":"5.50"}}
```

---

## API reference

The server runs at `http://localhost:3001`.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Server status, spec count, event count, divergence count |
| GET | `/specs` | All loaded specifications with health status |
| POST | `/verify/:spec` | Verify a single event against the spec |
| POST | `/verify/:spec/batch` | Verify a JSON array of events |
| GET | `/specs/:spec/divergences` | All unresolved divergences for a spec |
| DELETE | `/specs/:spec/divergences/:id` | Mark a divergence resolved |
| POST | `/simulate` | Portfolio simulation (old spec, new spec, portfolio) |
| POST | `/diff` | Financial logic diff between two spec texts |
| GET | `/audit` | Full audit trail, newest first |
| GET | `/specs/:spec/audit` | Spec-scoped audit trail |
| GET | `/specs/:spec/audit/export` | FCA-ready audit pack (JSON) |
| POST | `/specs/:spec/signoff` | Compliance sign-off, creates immutable audit entry |
| GET | `/events/recent` | Last 50 verification events |

---

## Architecture

PROOF is a Rust workspace with a Next.js dashboard.

crates/
proof-dsl/ Specification language: lexer, parser, typed AST
proof-eval/ Execution engine: deterministic spec evaluation
proof-verify/ Verification engine: spec vs system output diffing
proof-ingest/ Event ingestion: batch NDJSON, webhooks
proof-sim/ Simulation: portfolio replay and cohort analysis
proof-regulatory/ Regulatory rules: FCA Consumer Duty, FSCS
proof-audit/ Audit trail: SHA-256 spec hashing, FCA export
proof-server/ HTTP API: Axum + SQLite (WAL mode, sqlx migrations)
proof-cli/ Command-line interface

dashboard/ Next.js 14 + TypeScript + Tailwind CSS
regulatory/ Jurisdiction rule libraries (JSON)
examples/ Example .proof files and test portfolios


**Core pipeline:**

.proof file -> proof-dsl (parse) -> ProductSpec (AST)
|
proof-eval (evaluate against EvalInput)
|
proof-verify (compare vs system EvalOutput)
|
Divergence -> proof-audit (log to SQLite)
|
proof-server (REST API) -> dashboard (SWR polling)


**Data flow through the verification pipeline:**

Every event that enters PROOF carries both the input context (customer balance, event type, join date) and what the customer's system computed. PROOF evaluates the same input through the specification independently, then diffs the outputs field by field. Amount, rate, tier, and any custom fields are compared. Numeric deltas and percentages are computed. The result is either clean (no diff) or a structured divergence with full provenance.

**Persistence:**

All divergences, audit entries, and verification events are written to SQLite using sqlx with WAL journaling. The server rebuilds its in-memory state from the database on startup. Divergence resolution, sign-offs, and simulations are also logged. The audit log is append-only in practice and can be exported as an FCA audit pack for regulatory submission.

---

## Dashboard

Five screens:

**Dashboard** - Real-time verification feed. Every event processed by the server appears within 3 seconds via SWR polling. Red events are divergences. Clicking a divergence opens a detail panel showing the exact field mismatch, the spec's expected value, the system's actual value, and the delta.

**Specs** - All loaded specifications with current health status (clean or divergences active). Loaded automatically from the `PROOF_SPECS_DIR` directory on server startup.

**Simulate** - Portfolio impact simulation. Paste a proposed new spec, paste or upload a portfolio of customer events, and see exactly which customers are affected, by how much, and what FCA obligations the change triggers. Before you ship.

**Audit** - Complete immutable audit trail, persisted to SQLite. Every spec load, verify call, divergence, simulation, and sign-off is logged with the SHA-256 hash of the spec that was active at the time. Export an FCA-ready audit pack for any spec in one click.

**Diff** - Financial logic diff between two spec versions. Not a code diff. Shows changed tier thresholds, rate expressions, and obligation fields. Paste two `.proof` files and get a structured, human-readable diff of what changed financially.

---

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PROOF_DB` | `proof.db` | SQLite database path |
| `PROOF_SPECS_DIR` | `examples` | Directory to load .proof files from on startup |
| `RUST_LOG` | `proof_server=info` | Log level |
| `NEXT_PUBLIC_API_URL` | `http://localhost:3001` | API URL for the dashboard |

---

## Roadmap

**v0.2 - Bootstrapping**
Reverse-engineer existing Python, TypeScript, or Java interest calculation code into a draft `.proof` specification. Cold-start solution for onboarding existing products without writing specs from scratch.

**v0.3 - Integrations**
Kafka consumer for real-time event ingestion. EBA and CFPB regulatory rule libraries. PostgreSQL backend for production deployments.

**v0.4 - Distribution**
WASM build of the evaluation engine for zero-infrastructure browser demos. Authentication and multi-tenant support. Python SDK via PyO3.

**v1.0 - Intelligence**
Learn which spec patterns predict which error classes. Autonomous simulation: PROOF suggests which spec changes are high-risk before you propose them.

---

## Built with

- [Rust](https://www.rust-lang.org) - core engine, parser, HTTP server
- [Axum](https://github.com/tokio-rs/axum) - async HTTP framework
- [sqlx](https://github.com/launchbadge/sqlx) - async SQLite with compile-time query verification
- [Next.js 14](https://nextjs.org) - dashboard
- [Tailwind CSS](https://tailwindcss.com) - styling
- [SWR](https://swr.vercel.app) - real-time data fetching

---

## License

MIT