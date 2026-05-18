# Horoscope Panchang — objectives and progress

This document records **why** the project exists and **what has been implemented** so far.
For a quick run guide, see [`README.md`](https://github.com/kannandr/Horoscope/blob/main/README.md). For runtime shape, see
[`docs/platform-architecture.md`](https://github.com/kannandr/Horoscope/blob/main/docs/platform-architecture.md).

## Objective

Deliver a **temple-style Panchang calendar** (tithi / nakshatra transitions, hora,
muhurta) backed by a **deterministic, in-repo calculation core** with no
runtime ephemeris HTTP API. The product surface is a **Next.js / React UI**, a
**versioned HTTP API**, and an **MCP endpoint** so the same calculation core
serves humans, services, and language models.

## Architecture

| Layer       | Path                              | Purpose                                                       |
|-------------|-----------------------------------|---------------------------------------------------------------|
| Core        | `rust/crates/panchang-core`       | Sun/Moon ephemeris, ayanamsha, angas, boundaries, rise/set, hora, day segments, muhurta. |
| HTTP API    | `rust/crates/panchang-api`        | axum service on `/v1/panchang/*` and `/v1/muhurta/search`. OpenAPI doc. |
| MCP         | `rust/crates/panchang-mcp`        | JSON-RPC tools over the same core (private beta).             |
| Golden      | `rust/crates/panchang-golden`     | Snapshot fixtures and parity harness.                         |
| Web UI      | `web/`                            | Next.js 16 / React 19 calendar UI.                            |
| Infra       | `infra/bicep/`                    | Azure Container Apps deployment.                              |

The Next.js server proxies `/api/panchang/*` to the Rust API
(`PANCHANG_API_BASE_URL`, default `http://127.0.0.1:8080`). Address search and
reverse geocoding run inside the Next.js server (`tz-lookup` + Nominatim) so
the deterministic engine never touches network I/O.

## What is implemented

### Calculation core (`panchang-core`)

- Time core (UT ≈ UTC, Julian Day, local↔UTC).
- Tropical Sun/Moon: Meeus periodic terms ported from the Python reference; Surya-mean engine for sanity.
- Ayanamsha: Lahiri primary, Lahiri-alt stub, Raman.
- Angas: tithi (with paksha + day), nakshatra (+ Tamil + pada), yoga, karana, sun/moon rashi, vaara.
- Boundaries: tithi/karana/nakshatra/yoga start/end via bisection.
- Rise/set + hora (24 daytime/nighttime planetary horas).
- `day_segments`: per-civil-day tithi & nakshatra interval lists for calendar grids.
- Muhurta search: South Indian Tamil preset + scoring helper.

### HTTP API (`panchang-api`)

- `POST /v1/panchang/snapshot`
- `POST /v1/panchang/civil-day`
- `POST /v1/panchang/month`
- `POST /v1/muhurta/search`
- `GET /healthz`, `/readyz`, `/openapi.json`
- Request id middleware, structured JSON logs, permissive CORS.

### React UI (`web`)

- Toolbar: address input, browser geolocation, advanced (timezone, lat/lon, engine, ayanamsha).
- Tabs: Month, Week, Day, Auspicious Time.
- Day cell: tithi & nakshatra leaders, transitions filtered to the cell's local
  date with **green-start / red-end pill chips** (time-only since the date is
  the cell). Legend strip explains the colors.
- Daily view: snapshot details, transitions table, day & night hora tables.
- Muhurta view: ranked windows with reasons and exclusions.
- Server-side geocoding via Nominatim and `tz-lookup`.

## Known limits

- UT ≈ UTC (no ΔT table yet).
- Civil day boundaries are **local midnight → midnight**, not sunrise-bound
  panchanga days.
- Muhurta scoring is a heuristic preset, not a full Tamil panchanga rules engine.
- The Python reference engine has been retired; parity work continues via the
  Rust golden fixtures.

## Possible follow-ups

- Sunrise-anchored panchanga-day mode for cells.
- Larger golden fixture set covering DST transitions and high-latitude edge cases.
- More muhurta presets (Marriage, Griha pravesha, Aksharabhyasa, etc.).

*Last updated: post-Python migration; Rust + Next.js + MCP only.*
