# Horoscope Panchang

Native Panchang platform with a deterministic Rust calculation core (`panchang-core`),
a versioned HTTP API (`panchang-api`), an MCP endpoint (`panchang-mcp`), and a
Next.js / React UI (`web`). Azure Container Apps deployment lives under `infra/`.

## Run locally

Two processes:

```bash
# 1. Rust calculation API (default :8080)
cd rust
cargo run -p panchang-api

# 2. Next.js UI (default :3000)
cd ../web
npm ci
npm run dev
```

The UI calls `http://localhost:8080` by default. Override with
`PANCHANG_API_BASE_URL` if the API runs elsewhere.

Address search and reverse geocoding run inside the Next.js server using
OpenStreetMap Nominatim and `tz-lookup`. No external paid ephemeris APIs are
used for Panchang math — the engine is fully deterministic and in-repo.

## Tests

```bash
# Rust unit + golden tests
cd rust && cargo test --workspace
```

## Deploy (Azure)

Infrastructure is in `infra/bicep/`. GitHub Actions workflow **platform** (`.github/workflows/platform.yml`) builds three container images in Azure Container Registry, then deploys Azure Container Apps.

**This environment cannot run `az` for you**—deploy from GitHub or your own machine with [Azure CLI](https://learn.microsoft.com/cli/azure/install-azure-cli) installed.

### One-time setup

1. In Azure, pick a **resource group** name, **region** (e.g. `westus3`), and a **globally unique** ACR short name (alphanumeric, e.g. `panchangstgacr`).
2. In the GitHub repo, add **Actions variables**: `AZURE_RESOURCE_GROUP`, `AZURE_REGISTRY_NAME`, and optionally `AZURE_LOCATION`.
3. Add **Actions secrets** for workload identity / service principal login (`AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_SUBSCRIPTION_ID`) and MCP access (`MCP_SHARED_SECRET`). Full tables are in [`infra/README.md`](infra/README.md).
4. Grant that identity permission on the subscription or resource group (e.g. Contributor).

### Run deploy

Open **Actions → platform → Run workflow**. Only the **workflow_dispatch** job deploys; merges to `main` still run tests only.

After deployment, the **web** URL is in the ARM deployment outputs (`webUrl`; see `infra/README.md`). **Web is public HTTPS**. **MCP is public HTTPS but requires the shared GUID password** on `/mcp` using either `Authorization: Bearer <MCP_SHARED_SECRET>` or `x-mcp-password: <MCP_SHARED_SECRET>`.

See [`infra/README.md`](infra/README.md) for CLI equivalents and parameter details.
