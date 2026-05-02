# Panchang Platform Architecture

## Runtime Shape

The platform is a deterministic Rust calculation core fronted by a versioned
HTTP API, a hosted MCP endpoint, and a Next.js UI:

- `panchang-core`: deterministic Rust calculation library.
- `panchang-api`: versioned HTTP API for UI and external service callers.
- `panchang-mcp`: hosted MCP endpoint exposing model tools over JSON-RPC.
- `web`: Next.js UI that calls the API through route handlers; address search
  and reverse geocoding are handled inside the Next.js server with
  `tz-lookup` and OpenStreetMap Nominatim.
- `infra`: Azure Container Apps deployment.

The Python reference implementation has been retired; Rust is the only
calculation engine.

## API Boundary

The UI and MCP server must not duplicate Panchang logic. They call the same Rust
core service functions through shared request/response structs:

- Snapshot calculations for a local datetime.
- Civil-day tithi/nakshatra segmentation.
- Month cell precomputation.
- South Indian/Tamil-focused auspicious-window search.

## Rust Calculation Coverage

The Rust `panchang-core` crate is the calculation engine:

- `time.rs` — time core (UT, Julian day, local↔UTC).
- `ephemeris.rs` + `meeus_tables.rs` — apparent tropical Sun/Moon (Meeus periodic terms) and a Surya-mean alternative.
- `ayanamsha.rs` — Lahiri, Lahiri-alt stub, Raman.
- `names.rs` — angas / weekday name tables.
- `angas.rs` — tithi, nakshatra (+ Tamil + pada), yoga, karana, rashi, vaara.
- `boundaries.rs` — tithi / karana / nakshatra / yoga start/end via bisection.
- `rise_set.rs` — sunrise / sunset / next sunrise.
- `hora.rs` — 24 day/night planetary horas.
- `day_segments.rs` — per-local-civil-day tithi / nakshatra intervals for grid views.
- `muhurta.rs` — auspicious-window search with presets.
- `lib.rs` — snapshot orchestration.

Location services remain outside the deterministic engine because geocoding is
network I/O, not Panchang calculation. They live in the Next.js server in
`web/app/lib/location.ts`.

## Hosted deployment security

Azure Container Apps deployment (`infra/bicep`) runs **without** Container Apps easy auth:

- **Web** and **MCP** use public HTTPS ingress; anyone can reach them unless you add your own protection (API gateway, IP rules, app-level auth).
- **API** stays on **internal** ingress only (called by the Next.js server using `PANCHANG_API_BASE_URL`).

No application database is introduced in v1.

## Open Source Boundary

The intended future open-source units are:

- `rust/crates/panchang-core`
- `rust/crates/panchang-mcp`
- Golden fixture tooling and protocol examples.

Azure deployment wiring and hosted operations remain outside the OSS boundary
unless intentionally released later.
