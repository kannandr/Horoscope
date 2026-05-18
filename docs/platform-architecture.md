# Panchang Platform Architecture

## Two layers, by intent

The platform is split into a **calculation core** and **usage apps that
consume the core**. The core answers "what is the Panchang at time T at
place P?". Usage apps decide what to do with that answer.

Current implementation status:

- Calendar/Panchang calculation is in Rust and local.
- MCP exposes calculation tools only.
- Auspicious-time scoring lives in `muhurta-engine` / `muhurta-api`, with an
  optional natural-language layer in the web app (`muhurtaParse` +
  `AuspiciousView`).

### Calculation core (the "MCP engine")

- `panchang-core` (Rust lib) — deterministic Panchang calculations.
- `panchang-api` (Rust bin) — versioned HTTP surface for usage apps and
  internal callers. Internal-only ingress in production.
- `panchang-mcp` (Rust bin) — JSON-RPC MCP surface for model/tool callers.
  Public HTTPS, gated by `MCP_SHARED_SECRET`.
- `horoscope-core` (Rust lib) + `horoscope-mcp` (Rust bin) — South Indian natal
  chart + Vimshottari dasha–bhukti; same MCP auth and JSON-RPC envelope as
  `panchang-mcp`. See [`docs/horoscope-mcp.md`](https://github.com/kannandr/Horoscope/blob/main/docs/horoscope-mcp.md).

### Usage apps (consume the core)

- `web` (Next.js) — Panchang viewer with month / week / day calendars.
- `muhurta-engine` (Rust lib) — auspicious-time rule + (future) model
  scoring. Uses `McpPanchangClient` to call `panchang-mcp` when
  `PANCHANG_MCP_BASE_URL` is set (production / recommended local).
  `InProcessPanchangClient` uses `panchang-core` directly only when that
  URL is unset (fast local fallback).
- `muhurta-api` (Rust bin) — HTTP surface for the muhurta usage app
  (`POST /v1/muhurta/search`).

```
panchang-core (lib)
  ├── panchang-api  (HTTP /v1/panchang/*)
  ├── panchang-mcp  (JSON-RPC /mcp)
  ├── horoscope-core (lib) ──> panchang-core
  │       └── horoscope-mcp (JSON-RPC /mcp)
  └── (types + math reused by muhurta-engine for JD / ISO helpers)

muhurta-engine (lib) ──JSON-RPC──> panchang-mcp
       │
       └── muhurta-api  (HTTP /v1/muhurta/*)

usage apps:
  web ──> panchang-api  (calendar / day / snapshot)
  web ──> muhurta-api   (auspicious search)
  agents/models ──> panchang-mcp  (Panchang calculation tools)
  agents/models ──> horoscope-mcp (natal chart tool)
```

## Why split muhurta from the core

Muhurta scoring is one *use case* of the Panchang core, not a part of the
core itself. Keeping it separate:

- Lets the calculation core stay pure: no rule tables, no presets, no
  scoring weights, no future ML model living next to ephemeris math.
- Makes the auspicious calculator a real client/server boundary so a
  learned model can train against the same MCP surface that humans inspect.
- Lets us swap or fork the rule set, or run multiple usage apps
  (south-Indian general, north-Indian, travel-only, …) against one core.

## API Boundary

The web UI and any model/agent **must not** duplicate Panchang logic.

- For Panchang answers (snapshot / civil-day / panchang-day / month):
  call `panchang-api` (`/v1/panchang/*`) or `panchang-mcp` (JSON-RPC tools
  `calculate_panchang_*`, `list_civil_day_segments`,
  `list_inauspicious_periods`).
- For auspicious-window answers: call `muhurta-api`
  (`POST /v1/muhurta/search`). With `PANCHANG_MCP_BASE_URL` set (Azure and
  recommended local), it loads Panchang answers **only** via
  `panchang-mcp` JSON-RPC — same tools agents use. Without that URL it falls
  back to in-process core math for developer convenience.

- For South Indian natal chart JSON (lagna, grahas Sun/Moon, dasha–bhukti):
  call **`horoscope-mcp`** (`calculate_south_indian_natal_chart`). Details:
  [`docs/horoscope-mcp.md`](https://github.com/kannandr/Horoscope/blob/main/docs/horoscope-mcp.md).

The Panchang MCP server deliberately does **not** expose muhurta scoring tools.
That keeps MCP suitable for open-source release as a calculation layer, while
event-specific rule packs, personalization, and natural-language orchestration
can evolve independently in the usage layer.

## Rust calculation coverage

`panchang-core` is the calculation engine:

- `time.rs` — UT, Julian day, local↔UTC, ISO formatting.
- `ephemeris.rs` + `meeus_tables.rs` — apparent tropical Sun/Moon (Meeus
  periodic terms) and a Surya-mean alternative.
- `ayanamsha.rs` — Lahiri, Lahiri-alt stub, Raman.
- `names.rs` — angas / weekday / Tamil month + year tables.
- `angas.rs` — tithi, nakshatra (+ Tamil + pada), yoga, karana, rashi, vaara.
- `boundaries.rs` — tithi / karana / nakshatra / yoga start/end via bisection.
- `rise_set.rs` — sunrise / sunset / next sunrise.
- `hora.rs` — 24 day/night planetary horas.
- `day_segments.rs` — civil_day, panchang_day, month, Tamil calendar,
  Rahu/Yama/Gulika, Abhijit.
- `planets_jpl.rs` — JPL approximate Kepler ephemeris (Table 1, ~1800–2050 AD):
  geocentric Mercury–Saturn apparent ecliptic longitude + mean lunar north node;
  used by `horoscope-core` for sidereal navagraha (minus `ayanamsha`).
- `lib.rs` — snapshot orchestration.

Auspicious-time logic is **not** here anymore; it lives in
`muhurta-engine`.

Location services (Nominatim geocode + tz-lookup) stay outside the
deterministic engine because geocoding is network I/O. They live in the
Next.js server in `web/app/lib/location.ts`.

## Hosted deployment security

Azure Container Apps deployment (`infra/bicep`) runs **without** Container
Apps easy auth:

- **Web** uses public HTTPS ingress.
- **MCP** uses public HTTPS ingress plus an app-level shared GUID password
  on `/mcp`. Clients pass it as `Authorization: Bearer <MCP_SHARED_SECRET>`
  or `x-mcp-password: <MCP_SHARED_SECRET>`. Intentionally simple abuse
  resistance, not full identity management.
- **horoscope-mcp** uses the same shared-secret pattern on its own public `/mcp`.
- **panchang-api** stays on **internal** ingress (called by the Next.js
  server using `PANCHANG_API_BASE_URL`).
- **muhurta-api** stays on **internal** ingress (called by the Next.js
  server using `MUHURTA_API_BASE_URL`). In wire mode it calls MCP using
  `PANCHANG_MCP_BASE_URL` plus `MCP_SHARED_SECRET`.

No application database is introduced in v1.

## Third-party boundaries (calculations stay local)

- **Ephemeris, angas, sunrise/sunset, hora, Tamil calendar segments**:
  computed **only** inside `panchang-core` (Rust), reached via
  `panchang-api` or `panchang-mcp`.
- **Muhurta scoring**: computed locally in `muhurta-engine` (Rust), reached
  via `muhurta-api`. In production-style wiring, `muhurta-engine` obtains
  Panchang data from `panchang-mcp`; it does not call external horoscope,
  astrology, or calendar SaaS.
- **Optional network I/O**: address search / reverse geocode uses
  OpenStreetMap **Nominatim** (`web/app/lib/location.ts`) to resolve
  **place names to latitude, longitude, and timezone**. That is geography
  lookup only; Panchang payloads are not sent to third parties.

## Open Source Boundary

The intended future open-source units are:

- `rust/crates/panchang-core`
- `rust/crates/panchang-mcp`
- `rust/crates/muhurta-engine` (with rule tables that can fork)
- Golden fixture tooling and protocol examples.

- `rust/crates/horoscope-core`
- `rust/crates/horoscope-mcp`

Azure deployment wiring and hosted operations remain outside the OSS
boundary unless intentionally released later.
