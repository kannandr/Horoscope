# Azure Deployment

This folder contains the Container Apps deployment.

## Services

- `web`: Next.js UI, **public** external ingress (Container Apps easy auth **disabled**).
- `api`: Rust HTTP API, **internal** ingress (reachable only from inside the Container Apps environment, e.g. from `web`).
- `mcp`: Panchang MCP at **`/mcp`**, **public** external ingress with an app-level shared GUID password.
- `horoscope-mcp`: Natal-chart MCP at **`/mcp`**, **public** external ingress, same shared-secret headers as `mcp`.
- `muhurta`: Rust HTTP API for auspicious-time search, **internal** ingress.

## Required parameters (Bicep)

- `acrName`: **short name** of a Container Registry that already exists in the same resource group (5–50 alphanumeric characters, globally unique in Azure). The GitHub Action creates the registry on first run if it is missing.
- `webImage`, `apiImage`, `mcpImage`, `muhurtaImage`, `horoscopeImage`: full ACR image references (e.g. `myregistry.azurecr.io/panchang-web:abc123`).
- `mcpSharedSecret`: shared password stored as a Container Apps secret and exposed to **both** MCP containers as `MCP_SHARED_SECRET`.

The deployment intentionally creates no application database. State is limited to
Container Apps runtime configuration, logs, metrics, and disposable in-process caches.

## GitHub Action bootstrap

The `platform` workflow (manual `workflow_dispatch`):

1. Creates the resource group and ACR if they do not exist.
2. Builds the images in ACR with `az acr build` (`panchang-api`, `panchang-mcp`, `horoscope-mcp`, `muhurta-api`, `panchang-web`).
3. Deploys `infra/bicep/main.bicep`, which references the **existing** registry and provisions Log Analytics, App Insights, the Container Apps environment, and the apps.

Set repository **Variables** (Settings → Secrets and variables → Actions → Variables):

| Variable | Example | Purpose |
| -------- | ------- | ------- |
| `AZURE_RESOURCE_GROUP` | `rg-panchang-stg` | Target resource group |
| `AZURE_REGISTRY_NAME` | `panchangstgacr` | ACR short name (must be globally unique) |
| `AZURE_LOCATION` | `westus3` | Azure region (optional; defaults in workflow) |

Set **Secrets** for federated identity login (deploy job only):

| Secret | Purpose |
| ------ | ------- |
| `AZURE_CLIENT_ID` | Service principal (or app) for OIDC / `az login` |
| `AZURE_TENANT_ID` | Entra tenant |
| `AZURE_SUBSCRIPTION_ID` | Azure subscription |
| `MCP_SHARED_SECRET` | Shared GUID password required by `/mcp` |

Configure the Azure service principal with both roles on the target resource group:

- `Contributor` to create/update Azure resources.
- `User Access Administrator` to let Bicep assign `AcrPull` to the Container Apps managed identity.

## Deploy

In GitHub: **Actions** → **platform** → **Run workflow**, and set the `environment` input (default `stg`).

The deploy job runs only on `workflow_dispatch` (not on every push). Pushes still run **test** (Rust + Next build).

## Outputs (after deploy)

From the deployment in the Azure portal, or:

```bash
az deployment group show -g "$RG" -n <deployment-name> --query properties.outputs -o json
```

Bicep outputs include `webUrl`, `mcpUrl`, and `horoscopeMcpUrl`.

## Local / CLI deploy (same Bicep)

Create the group and registry, build and push images, then:

```bash
az deployment group create \
  --resource-group "$AZURE_RESOURCE_GROUP" \
  --template-file infra/bicep/main.bicep \
  --parameters \
    environmentName=stg \
    location=westus3 \
    acrName="$AZURE_REGISTRY_NAME" \
    webImage="${AZURE_REGISTRY_NAME}.azurecr.io/panchang-web:TAG" \
    apiImage="${AZURE_REGISTRY_NAME}.azurecr.io/panchang-api:TAG" \
    mcpImage="${AZURE_REGISTRY_NAME}.azurecr.io/panchang-mcp:TAG" \
    muhurtaImage="${AZURE_REGISTRY_NAME}.azurecr.io/muhurta-api:TAG" \
    horoscopeImage="${AZURE_REGISTRY_NAME}.azurecr.io/horoscope-mcp:TAG" \
    mcpSharedSecret="$MCP_SHARED_SECRET"
```
