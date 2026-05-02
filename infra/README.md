# Azure Deployment

This folder contains the Container Apps deployment for the private beta.

## Services

- `web`: Next.js UI, external ingress, Microsoft Entra login required.
- `api`: Rust HTTP API, internal ingress.
- `mcp`: hosted remote MCP endpoint at `/mcp`, external ingress, Microsoft Entra bearer auth required.

## Required parameters (Bicep)

- `acrName`: **short name** of a Container Registry that already exists in the same resource group (5–50 alphanumeric characters, globally unique in Azure). The GitHub Action creates the registry on first run if it is missing.
- `webImage`, `apiImage`, `mcpImage`: full ACR image references (e.g. `myregistry.azurecr.io/panchang-web:abc123`).
- `entraTenantId`: tenant for private beta sign-in.
- `webClientId`, `webClientSecret`: Entra app registration used by the UI Container App auth sidecar.
- `mcpClientId`, `mcpClientSecret`: Entra app registration used by the MCP Container App auth sidecar.

The deployment intentionally creates no application database. State is limited to
Container Apps runtime configuration, logs, metrics, and disposable in-process caches.

## GitHub Action bootstrap

The `platform` workflow (manual `workflow_dispatch`):

1. Creates the resource group and ACR if they do not exist.
2. Builds the three images in ACR with `az acr build`.
3. Deploys `infra/bicep/main.bicep`, which references the **existing** registry and provisions Log Analytics, App Insights, the Container Apps environment, and the three apps.

Set repository **Variables** (Settings → Secrets and variables → Actions → Variables):

| Variable | Example | Purpose |
| -------- | ------- | ------- |
| `AZURE_RESOURCE_GROUP` | `rg-panchang-stg` | Target resource group |
| `AZURE_REGISTRY_NAME` | `panchangstgacr` | ACR short name (must be globally unique) |
| `AZURE_LOCATION` | `westus3` | Azure region (optional; defaults in workflow) |

Set **Secrets** for federated identity login and Entra:

| Secret | Purpose |
| ------ | ------- |
| `AZURE_CLIENT_ID` | Service principal (or app) for OIDC / `az login` |
| `AZURE_TENANT_ID` | Entra tenant |
| `AZURE_SUBSCRIPTION_ID` | Azure subscription |
| `ENTRA_TENANT_ID` | Same as tenant (or app’s tenant) for Container Apps easy auth |
| `WEB_AUTH_CLIENT_ID` / `WEB_AUTH_CLIENT_SECRET` | App registration for the **web** Container App |
| `MCP_AUTH_CLIENT_ID` / `MCP_AUTH_CLIENT_SECRET` | App registration for **MCP** easy auth |

Configure the Azure service principal with access to the subscription (e.g. Contributor on the resource group) and with permission to create resources in that region.

## Deploy

In GitHub: **Actions** → **platform** → **Run workflow**, and set the `environment` input (default `stg`).

The deploy job runs only on `workflow_dispatch` (not on every push). Pushes still run **test** (Rust + Next build).

## Outputs (after deploy)

From the deployment in the Azure portal, or:

```bash
az deployment group show -g "$RG" -n <deployment-name> --query properties.outputs -o json
```

Bicep outputs include `webUrl` and `mcpUrl`.

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
    entraTenantId="$TENANT" \
    webClientId="$WEB_ID" \
    webClientSecret="$WEB_SECRET" \
    mcpClientId="$MCP_ID" \
    mcpClientSecret="$MCP_SECRET"
```