# Horoscope Panchang

Native Panchang platform with a deterministic Rust calculation core (`panchang-core`),
a versioned HTTP API (`panchang-api`), MCP endpoints (`panchang-mcp` for Panchang and
`horoscope-mcp` for South Indian natal charts), and a separate auspicious-time usage app (`muhurta-engine` + `muhurta-api`). The
browser experience is a Next.js / React UI (`web`). Azure Container Apps
deployment lives under `infra/`.

Useful docs:

- Architecture: [`docs/platform-architecture.md`](docs/platform-architecture.md)
- Rust engine review: [`docs/rust-engine.md`](docs/rust-engine.md)
- MCP integration (Panchang + Horoscope): [`docs/mcp.md`](docs/mcp.md), [`docs/horoscope-mcp.md`](docs/horoscope-mcp.md)
- South Indian **natal horoscope MCP** (plan + JSON contract): [`docs/south-indian-horoscope-mcp-plan.md`](docs/south-indian-horoscope-mcp-plan.md)
- Local muhurta-agent roadmap: [`docs/panchang-engine-and-local-muhurta-agent.md`](docs/panchang-engine-and-local-muhurta-agent.md)

## Run locally

The platform is split into a calculation core and usage apps that consume it.
For the calendar UI only, start `panchang-api` and `web`. For the auspicious
time tab, also start `muhurta-api`. For the production-style wire boundary,
run `panchang-mcp` and point `muhurta-api` at it.

```bash
# 1. Panchang calculation core (default :8080)
cd rust
cargo run -p panchang-api

# 2. Muhurta usage app (default :8090) — auspicious-time scoring / Auspicious tab
cargo run -p muhurta-api

# 3. Next.js UI (default :3000)
cd ../web
npm ci
npm run dev
```

The UI talks to two upstreams:

- `PANCHANG_API_BASE_URL` (default `http://localhost:8080`) for calendar /
  day / snapshot data.
- `MUHURTA_API_BASE_URL` (default `http://localhost:8090`) for
  auspicious-time search. This is optional if you are only using the calendar
  views.

### Horoscope MCP (natal chart)

South Indian sidereal natal chart JSON (`schema_version` **1.2.0**) — lagna, navagraha sidereal longitudes,
Panchang at birth, Tamil hints, Vimshottari dasha–bhukti — is served by **`horoscope-mcp`**
(same JSON-RPC pattern as `panchang-mcp`). Run it on a free port (defaults clash with `panchang-api` / `panchang-mcp` on **8080**):

```bash
BIND_ADDR=0.0.0.0:8790 cargo run -p horoscope-mcp
```

See [`docs/horoscope-mcp.md`](docs/horoscope-mcp.md) and [`docs/south-indian-horoscope-mcp-plan.md`](docs/south-indian-horoscope-mcp-plan.md).

### Phase 2: Muhurta calls Panchang via MCP (recommended)

`muhurta-api` should **not** use the in-process `panchang-core` fallback in
environments where you care about the real wire boundary. Set:

- `PANCHANG_MCP_BASE_URL` — MCP service root, e.g. `http://127.0.0.1:8787`
  (no `/mcp` suffix; the client appends it).
- `MCP_SHARED_SECRET` — same shared password as `panchang-mcp`, passed as
  `Authorization: Bearer …`. Omit both locally if `panchang-mcp` runs
  without `MCP_SHARED_SECRET`.

Because `panchang-api` and `panchang-mcp` both default to port **8080**, run
MCP on another port when using the wire path:

```bash
# Terminal A — calendar HTTP API
cd rust && cargo run -p panchang-api

# Terminal B — MCP JSON-RPC (Phase 2 upstream for muhurta)
BIND_ADDR=0.0.0.0:8787 cargo run -p panchang-mcp

# Terminal C — muhurta-api (wire mode)
PANCHANG_MCP_BASE_URL=http://127.0.0.1:8787 cargo run -p muhurta-api

# Terminal D — web
cd ../web && npm run dev
```

If `PANCHANG_MCP_BASE_URL` is unset, `muhurta-api` logs a warning and falls
back to in-process core math (fast local iteration only). With
`PANCHANG_MCP_BASE_URL` set, `muhurta-api` calls the same MCP tools that a
future local natural-language agent will use.

`panchang-mcp` remains optional for callers that hit JSON-RPC directly; the
web calendar continues to use `panchang-api` only.

### Refresh a service after pulling new commits

The Rust binaries are long-running. After you pull commits that add or
change routes, the previously built binary still serves stale routes.
Rebuild + restart in one step:

```bash
# panchang-api on :8080 (default)
scripts/restart-api.sh

# horoscope-mcp on :8790 (example)
scripts/restart-api.sh horoscope-mcp
```

The script kills any process on the relevant port, builds the crate in
release mode, restarts the new binary, and waits for `/healthz` before
returning. Override the port via `PORT=…` and the log path via
`RESTART_LOG=…`.

Address search and reverse geocoding run inside the Next.js server using
OpenStreetMap Nominatim and `tz-lookup`. No external paid ephemeris APIs are
used for Panchang math — the engine is fully deterministic and in-repo.
Muhurta scoring is local Rust rule logic in `muhurta-engine`; there is no
hosted model dependency today.

## Tests

```bash
# Rust unit + golden tests
cd rust && cargo test --workspace
```

## Deploy (Azure)

Infrastructure is in `infra/bicep/`. GitHub Actions workflow **platform** (`.github/workflows/platform.yml`) builds five Rust/web container images in Azure Container Registry (`panchang-api`, `panchang-mcp`, `horoscope-mcp`, `muhurta-api`, `panchang-web`), then deploys Azure Container Apps.

**This environment cannot run `az` for you**—deploy from GitHub or your own machine with [Azure CLI](https://learn.microsoft.com/cli/azure/install-azure-cli) installed.

### One-time setup

1. In Azure, pick a **resource group** name, **region** (e.g. `westus3`), and a **globally unique** ACR short name (alphanumeric, e.g. `panchangstgacr`).
2. In the GitHub repo, add **Actions variables**: `AZURE_RESOURCE_GROUP`, `AZURE_REGISTRY_NAME`, and optionally `AZURE_LOCATION`.
3. Add **Actions secrets** for workload identity / service principal login (`AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_SUBSCRIPTION_ID`) and MCP access (`MCP_SHARED_SECRET`). Full tables are in [`infra/README.md`](infra/README.md).
4. Grant that identity `Contributor` plus `User Access Administrator` on the target resource group. `User Access Administrator` is needed because the Bicep template assigns `AcrPull` to the Container Apps managed identity.

### Run deploy

Open **Actions → platform → Run workflow**. Only the **workflow_dispatch** job deploys; merges to `main` still run tests only.

After deployment, the **web** URL is in the ARM deployment outputs (`webUrl`; see `infra/README.md`). **Web is public HTTPS**. **MCP is public HTTPS but requires the shared GUID password** on `/mcp` using either `Authorization: Bearer <MCP_SHARED_SECRET>` or `x-mcp-password: <MCP_SHARED_SECRET>`.

See [`infra/README.md`](infra/README.md) for CLI equivalents and parameter details.
