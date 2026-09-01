# PROOF

**Financial logic, verified pure.**

PROOF is an open-source financial logic verification platform.
It makes it impossible to silently ship wrong financial logic.

## What it does

1. **Specify** - write your product rules in the PROOF specification language
2. **Verify** - PROOF runs alongside your system in production, independently recomputing every calculation
3. **Simulate** - before deploying a change, replay 12 months of real portfolio history through the new spec and see the exact impact
4. **Audit** - every spec version is cryptographically signed; every verification event is logged; pull a regulator-ready audit pack in one command

## Quick start

```bash
# Check a spec evaluates correctly
proof check examples/savings-account.proof \
  --input '{"balance": 9840, "event": "daily_accrual"}'

# Compare your system's output against the spec
proof verify examples/savings-account.proof \
  --event system_output.json

# Simulate a spec change across your portfolio
proof simulate examples/savings-account.proof \
  --new examples/savings-account-v2.proof \
  --portfolio portfolio_events.ndjson
```

## Project structure

```
crates/
  proof-dsl/        Specification language - lexer, parser, AST
  proof-eval/       Execution engine - evaluates specs against inputs
  proof-verify/     Verification engine - diffs spec vs system output
  proof-ingest/     Event ingestion - Kafka, webhooks, batch files
  proof-sim/        Simulation engine - portfolio replay
  proof-regulatory/ Regulatory knowledge base - FCA, PRA, CFPB, EBA
  proof-audit/      Audit trail - cryptographic signing, immutable logs
  proof-server/     HTTP control API (Axum)
  proof-cli/        Command-line interface

python/proof/       Python SDK
dashboard/          Next.js dashboard
regulatory/         Jurisdiction-specific rule libraries
examples/           Example .proof specification files
```

## Stack

- **Core**: Rust (all hard logic)
- **SDK**: Python + PyO3/maturin
- **Dashboard**: Next.js + TypeScript
- **Events**: Kafka
- **Storage**: SQLite → PostgreSQL
- **CI**: GitHub Actions
