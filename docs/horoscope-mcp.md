# Horoscope MCP Server (`horoscope-mcp`)

Separate MCP binary for **South Indian sidereal natal charts**. It shares the same
HTTP JSON-RPC pattern as [`docs/mcp.md`](mcp.md) (`POST /mcp`, `initialize`,
`tools/list`, `tools/call`, `structuredContent`), and the same optional
`MCP_SHARED_SECRET` guard.

Design reference: [`docs/south-indian-horoscope-mcp-plan.md`](south-indian-horoscope-mcp-plan.md).

## Tool

| Name | Purpose |
|------|---------|
| `calculate_south_indian_natal_chart` | Birth place + local datetime → JSON `south_indian_natal_chart` (**schema_version** `1.2.0`): lagna; **navagraha** (Sun–Saturn sidereal + mean lunar nodes); Panchang at birth; Tamil hints; Vimshottari dasha–bhukti for **birth … as-of + `dasha_horizon_years`**. `frame` records `lunar_node_policy` and `slow_planet_ephemeris`. |

## Implementation

- **`horoscope-core`** — assembles the chart on top of **`panchang-core`** (`snapshot`, `panchang_day`, Sun/Moon ephemeris, JPL Kepler slow planets in `planets_jpl`, ascendant).
- **`horoscope-mcp`** — thin Axum adapter (mirror `panchang-mcp`).

## Local run

Default **`BIND_ADDR`** is `0.0.0.0:8080`, which conflicts with `panchang-api` / `panchang-mcp`. Use a free port:

```bash
cd rust
BIND_ADDR=0.0.0.0:8790 cargo run -p horoscope-mcp
```

Or:

```bash
scripts/restart-api.sh horoscope-mcp   # defaults to :8790
```

## Example `tools/call`

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "calculate_south_indian_natal_chart",
    "arguments": {
      "birth_local": "1990-08-15T14:32:00",
      "timezone": "Asia/Kolkata",
      "latitude": 13.0827,
      "longitude": 80.2707,
      "ayanamsha": "lahiri",
      "engine": "meeus",
      "dasha_horizon_years": 20,
      "as_of_local": "2026-05-02T12:00:00"
    }
  }
}
```

Omit **`as_of_local`** to anchor the dasha window end on **now** in **`timezone`**.

## Docker

Build context is the repo root:

```bash
docker build -f rust/crates/horoscope-mcp/Dockerfile .
```

Azure deploy uses the same image tag pattern as `panchang-mcp`; see `.github/workflows/platform.yml`.
