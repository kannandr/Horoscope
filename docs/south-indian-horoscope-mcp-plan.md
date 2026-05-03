# Plan: South Indian horoscope MCP

Goal: offer **one MCP tool** (same HTTP JSON-RPC pattern as `panchang-mcp`) that,
given **birth place + birth local date/time**, returns a **single JSON document**
with South Indian–oriented sidereal horoscope data for that instant — suitable
for apps, agents, and archival storage.

Calculations stay **in-repo** (Rust); no third-party astrology SaaS.

---

## 1. Product scope

| In scope (phased) | Out of scope (initially) |
|---------------------|---------------------------|
| Sidereal frame (Lahiri default; Raman optional) | Commercial chart PDF/layout |
| Lagna (ascendant) degree → rashi / nakshatra / pada | Thousands of divisional charts |
| Sun & Moon positions (already in engine) | ML-generated readings |
| Panchang-style angas at birth (reuse snapshot) | Non–South-Indian chart styles |
| **Vimshottari Dasha–Bhukti** (mahadasha + antardasha) from birth through **as-of + 20 years** | Pratyantardasha (Sookshma) in v1 JSON (optional later) |
| Tamil naming where we already have tables | |

---

## 2. Architecture (recommended)

```
rust/crates/
  horoscope-core/     # natal math: lagna, graha longitudes, assemble HoroscopeBirthJson
  horoscope-mcp/      # Axum + JSON-RPC POST /mcp (mirror panchang-mcp layout)
```

- **`horoscope-core`** depends on **`panchang-core`** for time/JD, ayanamsha,
  existing Sun/Moon ephemeris, and **`snapshot`**-equivalent Panchang angas.
- **`horoscope-mcp`** is a thin adapter: auth header pattern identical to
  `panchang-mcp`, tools: `initialize`, `tools/list`, `tools/call`.
- **Alternative:** add tools to **`panchang-mcp`** (`calculate_south_indian_horoscope`).
  Prefer a **separate binary** if release cadence, scaling, or IAM differ; merge
  if you want one endpoint for customers.

**Transport:** fully support the **same HTTP MCP** you already standardized:
`POST …/mcp`, JSON-RPC 2.0, `structuredContent` mirroring the Rust struct JSON.

---

## 3. MCP surface (draft)

### Tool: `calculate_south_indian_natal_chart`

**Input (arguments)** — aligned with `SnapshotRequest` so clients already integrated with Panchang can reuse fields:

| Field | Type | Required |
|-------|------|----------|
| `birth_local` | string | yes — `YYYY-MM-DDTHH:MM:SS` or RFC3339 |
| `timezone` | string | yes — IANA |
| `latitude` | number | yes |
| `longitude` | number | yes |
| `ayanamsha` | string enum | no — default `lahiri` |
| `engine` | string enum | no — default `meeus` for Sun/Moon; extended grahas may use same engine id |
| `dasha_horizon_years` | number | no — default **`20`**: dasha tree runs from birth through **server as-of date + this many years** (see §4.1) |
| `as_of_local` | string | no — if set, RFC3339 or `YYYY-MM-DDTHH:MM:SS`; fixes the “to-date” anchor for reproducible outputs (defaults to **now** in `timezone`) |

**Output:** one JSON object matching **`south_indian_natal_chart`** schema below
(under `result.structuredContent` in MCP).

---

## 4. Draft JSON structure (`south_indian_natal_chart`)

Versioned top-level object. All angular longitudes in **sidereal degrees**
`[0, 360)` unless noted. Names follow existing snake_case + Tamil labels where
available.

**Dasha–bhukti (Tamil: Dhasha Bhukthi / Bukti):** the **`dasha_bhukti`** object
carries **Vimshottari mahadasha and antardasha (bhukti)** intervals whose spans
**intersect** the closed JD window **`[birth_jd_ut, horizon_end_jd_ut]`**, where
**`horizon_end`** is **`as_of` + `dasha_horizon_years`** (default **20** calendar
years after the tool’s “to-date” anchor — see **`window`** and §4.1). That yields
a timeline **from date of birth through twenty years past the as-of instant**.

```json
{
  "schema_version": "1.2.0",
  "kind": "south_indian_natal_chart",

  "birth": {
    "birth_local": "1990-08-15T14:32:00",
    "timezone": "Asia/Kolkata",
    "latitude": 13.0827,
    "longitude": 80.2707,
    "utc_iso": "1990-08-15T09:02:00Z",
    "jd_ut": 2448110.876389
  },

  "frame": {
    "ayanamsha": "lahiri",
    "ayanamsha_deg": 23.85,
    "engine": "meeus",
    "sidereal_zodiac": "tropical_minus_ayanamsha"
  },

  "lagna": {
    "sidereal_longitude_deg": 124.56,
    "rashi_index": 5,
    "rashi_name": "Simha",
    "rashi_name_tamil": "Simmam",
    "nakshatra_index": 11,
    "nakshatra_name": "Purva Phalguni",
    "nakshatra_name_tamil": "Pooram",
    "nakshatra_pada": 2
  },

  "grahas": {
    "sun": {
      "sidereal_longitude_deg": 118.2,
      "rashi_index": 4,
      "rashi_name": "Karka",
      "nakshatra_index": 10,
      "nakshatra_name": "Magha",
      "nakshatra_pada": 1,
      "retrograde": false
    },
    "moon": {
      "sidereal_longitude_deg": 255.4,
      "rashi_index": 9,
      "rashi_name": "Dhanu",
      "nakshatra_index": 20,
      "nakshatra_name": "Purva Ashadha",
      "nakshatra_pada": 4,
      "retrograde": false
    },
    "mars": { "availability": "planned", "sidereal_longitude_deg": null },
    "mercury": { "availability": "planned", "sidereal_longitude_deg": null },
    "jupiter": { "availability": "planned", "sidereal_longitude_deg": null },
    "venus": { "availability": "planned", "sidereal_longitude_deg": null },
    "saturn": { "availability": "planned", "sidereal_longitude_deg": null },
    "rahu": { "availability": "planned", "sidereal_longitude_deg": null },
    "ketu": { "availability": "planned", "sidereal_longitude_deg": null }
  },

  "panchang_at_birth": {
    "vaara": "Wednesday",
    "tithi_name": "…",
    "yoga_name": "…",
    "karana_name": "…",
    "paksha": "krishna",
    "sunrise_local": "…",
    "sunset_local": "…"
  },

  "tamil_calendar_hint": {
    "solar_month_name": "…",
    "solar_month_name_tamil": "…",
    "tamil_year_name": "…",
    "weekday_name_tamil": "…"
  },

  "dasha_bhukti": {
    "system": "vimshottari",
    "aliases": { "bhukti_en": "Antardasha", "bhukti_ta_hint": "Bukti / anthira thasa" },

    "lords_order": [
      "ketu", "venus", "sun", "moon", "mars", "rahu", "jupiter", "saturn", "mercury"
    ],

    "mahadasha_years": {
      "ketu": 7,
      "venus": 20,
      "sun": 6,
      "moon": 10,
      "mars": 7,
      "rahu": 18,
      "jupiter": 16,
      "saturn": 19,
      "mercury": 17
    },

    "moon_at_birth": {
      "nakshatra_index": 20,
      "nakshatra_name": "Purva Ashadha",
      "starting_mahadasha_lord": "venus",
      "balance_of_starting_mahadasha_at_birth_days": 412.7
    },

    "window": {
      "birth_jd_ut": 2448110.876389,
      "as_of_local": "2026-05-03T10:00:00",
      "as_of_jd_ut": 2460834.916667,
      "horizon_end_local": "2046-05-03T10:00:00",
      "horizon_end_jd_ut": 2468142.916667,
      "horizon_years_after_as_of": 20,
      "timezone": "Asia/Kolkata"
    },

    "mahadashas": [
      {
        "lord": "venus",
        "lord_display_en": "Venus",
        "lord_display_ta": "Velli",
        "start_jd_ut": 2447618.5,
        "end_jd_ut": 2454923.5,
        "start_local": "1988-…",
        "end_local": "2008-…",
        "antardashas": [
          {
            "lord": "venus",
            "lord_display_en": "Venus",
            "lord_display_ta": "Velli",
            "start_jd_ut": 2447618.5,
            "end_jd_ut": 2447646.8,
            "start_local": "1988-…",
            "end_local": "1988-…"
          },
          {
            "lord": "sun",
            "lord_display_en": "Sun",
            "lord_display_ta": "Nyayiru",
            "start_jd_ut": 2447646.8,
            "end_jd_ut": 2447657.3,
            "start_local": "1988-…",
            "end_local": "1988-…"
          }
        ]
      },
      {
        "lord": "sun",
        "lord_display_en": "Sun",
        "lord_display_ta": "Nyayiru",
        "start_jd_ut": 2454923.5,
        "end_jd_ut": 2457115.0,
        "start_local": "2008-…",
        "end_local": "2014-…",
        "antardashas": []
      }
    ],

    "notes": "Emit every mahadasha and antardasha whose interval intersects [birth_jd_ut, horizon_end_jd_ut]; clip endpoints at the window. Numbers above are illustrative — implement §4.1 precisely in Rust."
  },

  "extensions": {
    "navamsa_d9": null,
    "pratyantardasha": null,
    "notes": "Reserved for D9, Sookshma–Pratyantar, Ashtakavarga, etc."
  }
}
```

### 4.1 Dasha–bhukti rules (normative for implementers)

- **System:** standard **Vimshottari** (120-year cycle, nine lords, fixed year
  lengths as in `mahadasha_years`).
- **Anchor at birth:** lord is determined from **Moon’s nakshatra** at birth
  (each nakshatra maps to a starting mahadasha); **remaining balance** of that
  mahadasha at `birth_jd_ut` is proportional to how far the Moon has progressed
  within the nakshatra ( pada / longitude fraction ).
- **Antardasha (bhukti):** within each mahadasha, the nine lords run in the **same
  cyclic order**, starting with the **mahadasha lord**; each antara length =
  `(mahadasha_years[antara_lord] / 120) × mahadasha_length_in_days`.
- **Window:** include every mahadasha and nested antardasha interval that
  **intersects** `[birth_jd_ut, horizon_end_jd_ut]` where
  `horizon_end = as_of_jd_ut + dasha_horizon_years × 365.25` **calendar years**
  expressed in JD via the same UTC pipeline as Panchang (implementation should
  use **increment `as_of` by years in the birth timezone** or civil date add —
  document chosen rule in code comments; prefer **add 20 years to `as_of` local
  date at same clock time**, DST-safe via `chrono`).
- **Coverage:** this is exactly **dasha–bhukti positions from date of birth**
  through **`as_of` plus `dasha_horizon_years`** (default **20** years): trim only
  at those window edges, not at “today” unless **`as_of`** is **now**.
- **Tamil display names** for lords are informational (`lord_display_ta`); exact
  spelling can follow your UI glossary.

### Design notes

- **`birth`** echoes inputs and adds canonical **`jd_ut`** / **`utc_iso`** for
  reproducibility (same spirit as `SnapshotResponse.jd_ut`).
- **`lagna`** is the main **new** computation vs today’s Panchang snapshot:
  local sidereal time → ascendant ecliptic longitude → sidereal → rashi /
  nakshatra / pada (reuse `names` tables + degree slicing rules from `angas`).
- **`grahas.sun` / `grahas.moon`** can be populated from existing ephemeris +
  ayanamsha; **`availability`** documents phased rollout for **true node /
  mean node / slow planets**.
- **`panchang_at_birth`** can be a **trimmed projection** of current
  `SnapshotResponse` + `angas` so clients get “full horoscope context” without
  shipping the entire snapshot blob twice — or embed full snapshot under
  `raw_snapshot` if you prefer one source of truth.
- **`dasha_bhukti`** is the machine-readable **Dasha + Bhukti (Antardasha)**
  timeline required for South Indian horoscope products; horizon defaults to
  **as-of + 20 years** but is tunable via request fields.
- **`extensions`** keeps the contract forward-compatible for D9 /
  Pratyantardasha / Ashtakavarga without breaking clients.

---

## 5. Implementation phases

### Phase A — Contract + MCP shell (shallow data)

- Add **`horoscope-mcp`** with auth mirroring **`panchang-mcp`**.
- Implement **`calculate_south_indian_natal_chart`** returning JSON with:
  - `birth`, `frame`, **`lagna` computed**, **`grahas.sun/moon`** from core,
    **`panchang_at_birth`** from **`snapshot`** pipeline,
    **`tamil_calendar_hint`** from existing Tamil helpers where applicable.
- **`availability: "planned"`** for remaining grahas.

### Phase B — Vimshottari dasha–bhukti engine

- Implement **`vimshottari_timeline(birth_jd_ut, moon_sidereal_deg, as_of_jd_ut, horizon_years)`**
  in **`horoscope-core`** (pure Rust, deterministic).
- Populate **`dasha_bhukti`** per §4.1; golden tests vs hand-calculated spans for
  a few known charts.
- Wire request knobs **`dasha_horizon_years`** (default 20) and optional
  **`as_of_local`**.

### Phase C — Navagraha longitudes + divisional charts

- Extend ephemeris for Mars–Saturn (+ nodes); **Rahu/Ketu** policy documented in `frame`.
- **Navamsa (D9)** under `extensions.navamsa_d9` when ready.
- Optional **`pratyantardasha`** nested under each antardasha later.

---

## 6. Risks & mitigations

| Risk | Mitigation |
|------|------------|
| Lagna sensitivity to birth time | Document rounding; use same TZ rules as Panchang (`local_to_utc`). |
| Ephemeris accuracy for slow planets | Unit tests vs published almanacs; version **`engine`** in JSON. |
| Large JSON | Emit only MD/antara intervals intersecting the dasha window; optional `dasha_levels: ["mahadasha"]` later. |
| Dasha boundary disputes | Publish **`schema_version`** + explicit rules in §4.1; snapshot golden vectors. |

---

## 7. Next steps (checklist)

1. Freeze **`schema_version` `1.2.0`** (natal + navagraha + **`dasha_bhukti`**); bump minor when adding pratyantar/D9.
2. Implement **`lagna_sidereal_deg(jd, lat, lon, ay)`** in Rust; tests vs known charts.
3. Wire **`horoscope-mcp`** tool + **`structuredContent`** encoding.
4. Add **`docs/horoscope-mcp.md`** (customer-facing) linking to this plan.
5. Azure: optional fourth Container App **`horoscope-mcp`** or route behind API gateway.

---

This plan keeps **one fully supported MCP style** (HTTP JSON-RPC + tools + structured
content) while growing the **semantic contract** for South Indian natal data in a
single versioned JSON envelope.
