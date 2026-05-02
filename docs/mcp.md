# Panchang MCP Server

This document describes the hosted Panchang Model Context Protocol (MCP) server
for model/tool callers.

## Overview

The MCP server exposes deterministic Panchang calculations from the Rust
`panchang-core` engine. It does not contain a separate calculation
implementation. All tool calls go through the same Rust service functions used
by the HTTP API.

Current staging endpoint:

```text
https://panchang-stg-mcp.blueground-f706ec9e.westus3.azurecontainerapps.io/mcp
```

Health endpoints:

```text
GET /healthz
GET /readyz
```

## Authentication

The hosted `/mcp` endpoint is public HTTPS, but it requires a shared GUID
password when `MCP_SHARED_SECRET` is configured.

Use either header:

```http
Authorization: Bearer <MCP_SHARED_SECRET>
```

or:

```http
x-mcp-password: <MCP_SHARED_SECRET>
```

Do not commit the real GUID password to source control. Store it in GitHub
Actions as `MCP_SHARED_SECRET`; Azure Container Apps receives it as a Container
Apps secret.

Unauthenticated MCP calls return:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "MCP shared password is required"
  }
}
```

## Transport

The current server accepts JSON-RPC 2.0 over hosted HTTP at `POST /mcp`.
Stdio support is intentionally left for a later local tooling release.

Required headers:

```http
content-type: application/json
Authorization: Bearer <MCP_SHARED_SECRET>
```

## Tools

### `calculate_panchang_snapshot`

Calculates Panchang angas and local transition times for an observer and local
datetime.

Input:

```json
{
  "when_local": "2026-05-02T12:00:00",
  "timezone": "America/Los_Angeles",
  "latitude": 37.6821,
  "longitude": -121.768,
  "ayanamsha": "lahiri",
  "engine": "meeus"
}
```

Required fields:

```text
when_local, timezone, latitude, longitude
```

Optional fields:

```text
ayanamsha: lahiri | lahiri_alt_stub | raman
engine: meeus | surya_mean
```

### `list_civil_day_segments`

Lists tithi, nakshatra, yoga, and karana intervals that intersect one local
civil day. The civil day is local midnight to the next local midnight.

Input:

```json
{
  "date": "2026-05-02",
  "timezone": "America/Los_Angeles",
  "latitude": 37.6821,
  "longitude": -121.768,
  "ayanamsha": "lahiri",
  "engine": "meeus"
}
```

Required fields:

```text
date, timezone, latitude, longitude
```

Each returned interval includes the full astronomical boundary and a clipped
boundary for the requested day window:

```text
start_local/end_local: full interval boundary
clipped_start_local/clipped_end_local: interval portion inside the day
starts_before_window/ends_after_window: true when the full interval crosses the day boundary
```

### `calculate_panchang_day`

Calculates a richer Panchang day object for specialist Panchang and muhurta
work. It includes sunrise/sunset, vaara, angas at sunrise, tithi/nakshatra/
yoga/karana intervals, hora, rahu kalam, yama gandam, gulika kalam, and
abhijit muhurta. It also returns basic Tamil calendar metadata: solar month,
Tamil year name, ayana, ritu, and Tamil weekday label.

Input:

```json
{
  "date": "2026-05-02",
  "timezone": "America/Los_Angeles",
  "latitude": 37.6821,
  "longitude": -121.768,
  "day_mode": "sunrise_day",
  "ayanamsha": "lahiri",
  "engine": "meeus"
}
```

Required fields:

```text
date, timezone, latitude, longitude
```

Optional fields:

```text
day_mode: civil_midnight | sunrise_day
ayanamsha: lahiri | lahiri_alt_stub | raman
engine: meeus | surya_mean
```

Use `civil_midnight` for UI calendar cells. Use `sunrise_day` for traditional
Panchang and muhurta reasoning.

### `list_inauspicious_periods`

Returns only the daytime caution blocks from `calculate_panchang_day`:

```text
rahu_kalam
yama_gandam
gulika_kalam
```

Input is the same as `calculate_panchang_day`.

### `search_auspicious_windows`

Searches South Indian/Tamil-focused auspicious time windows with transparent
scoring reasons and exclusions.

Input:

```json
{
  "date_start": "2026-05-02",
  "date_end": "2026-05-04",
  "timezone": "America/Los_Angeles",
  "latitude": 37.6821,
  "longitude": -121.768,
  "purpose_preset": "general",
  "min_duration_minutes": 30,
  "ayanamsha": "lahiri",
  "engine": "meeus"
}
```

Required fields:

```text
date_start, date_end, timezone, latitude, longitude
```

### `explain_auspicious_window`

Explains the scoring fields returned by `search_auspicious_windows`.

Input:

```json
{
  "window": {
    "score": 12,
    "reasons": ["example positive rule"],
    "exclusions": ["example caution"]
  }
}
```

## JSON-RPC Examples

### Initialize

```bash
curl -s -X POST \
  "https://panchang-stg-mcp.blueground-f706ec9e.westus3.azurecontainerapps.io/mcp" \
  -H "content-type: application/json" \
  -H "Authorization: Bearer $MCP_SHARED_SECRET" \
  --data '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize"
  }'
```

### List Tools

```bash
curl -s -X POST \
  "https://panchang-stg-mcp.blueground-f706ec9e.westus3.azurecontainerapps.io/mcp" \
  -H "content-type: application/json" \
  -H "Authorization: Bearer $MCP_SHARED_SECRET" \
  --data '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }'
```

### Call Snapshot Tool

```bash
curl -s -X POST \
  "https://panchang-stg-mcp.blueground-f706ec9e.westus3.azurecontainerapps.io/mcp" \
  -H "content-type: application/json" \
  -H "Authorization: Bearer $MCP_SHARED_SECRET" \
  --data '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "calculate_panchang_snapshot",
      "arguments": {
        "when_local": "2026-05-02T12:00:00",
        "timezone": "America/Los_Angeles",
        "latitude": 37.6821,
        "longitude": -121.768,
        "ayanamsha": "lahiri",
        "engine": "meeus"
      }
    }
  }'
```

## Response Shape

Successful calculation tool calls return both text and structured content:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{...serialized calculation result...}"
      }
    ],
    "structuredContent": {
      "...": "same calculation result as JSON"
    }
  }
}
```

Validation and calculation failures return JSON-RPC errors:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "error": {
    "code": -32000,
    "message": "tool call failed",
    "data": {
      "message": "details from the Rust engine"
    }
  }
}
```

## Local Development

Run the MCP server locally:

```bash
cd rust
MCP_SHARED_SECRET=local-dev-secret cargo run -p panchang-mcp
```

Local endpoint:

```text
http://localhost:8080/mcp
```

If `MCP_SHARED_SECRET` is unset or empty, local `/mcp` calls are allowed without
the shared password. Hosted environments should always set it.

## Azure Deployment Notes

The GitHub Actions `platform` workflow deploys the MCP service to Azure
Container Apps:

- Container App: `panchang-stg-mcp`
- External ingress: enabled
- Target port: `8080`
- Secret: `MCP_SHARED_SECRET`
- Health checks: `/healthz`, `/readyz`

The MCP URL is emitted by Bicep as `mcpUrl`.

## Client Notes

- Treat `MCP_SHARED_SECRET` like a password, not like a public API key.
- Prefer `Authorization: Bearer <secret>` for model-hosted clients.
- Use `x-mcp-password` only when a client cannot set authorization headers.
- The server currently uses permissive CORS because it is a tool endpoint, not
  a browser application boundary.
- This shared-secret setup is simple abuse resistance for private beta. It is
  not per-user identity, rate limiting, or tenant isolation.

## Open Source Boundary

This document is safe to share because it does not include the real shared GUID
password. For a future open-source release, keep the same rule:

- publish the server code and usage examples;
- publish placeholders for secrets;
- keep hosted Azure configuration, real URLs, and real passwords private unless
  intentionally releasing a public beta endpoint.
