import "server-only";

const DEFAULT_PANCHANG_BASE = "http://localhost:8080";
const DEFAULT_MUHURTA_BASE = "http://localhost:8090";

export class UpstreamError extends Error {
  readonly status: number;
  readonly upstream: string;
  readonly upstreamBody: unknown;

  constructor(upstream: string, status: number, message: string, upstreamBody: unknown) {
    super(`${upstream} ${status}: ${message}`);
    this.name = "UpstreamError";
    this.status = status;
    this.upstream = upstream;
    this.upstreamBody = upstreamBody;
  }
}

async function postJson<T>(base: string, label: string, pathName: string, payload: unknown): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`${base}${pathName}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(payload),
      cache: "no-store"
    });
  } catch (e) {
    const cause = e instanceof Error ? e.message : String(e);
    throw new UpstreamError(label, 503, `unreachable (${cause})`, null);
  }
  if (!res.ok) {
    const text = await res.text();
    let parsed: unknown = text;
    let message = text;
    try {
      const j = JSON.parse(text);
      parsed = j;
      if (j && typeof j === "object" && typeof (j as { error?: unknown }).error === "string") {
        message = (j as { error: string }).error;
      }
    } catch {
      // not JSON; keep raw text
    }
    throw new UpstreamError(label, res.status, message, parsed);
  }
  return (await res.json()) as T;
}

/** Call the Panchang calculation core (panchang-api on :8080 by default). */
export async function callPanchang<T>(pathName: string, payload: unknown): Promise<T> {
  const base = process.env.PANCHANG_API_BASE_URL ?? DEFAULT_PANCHANG_BASE;
  return postJson<T>(base, "panchang-api", pathName, payload);
}

/**
 * Call the Muhurta usage app (muhurta-api on :8090 by default).
 *
 * Phase 2: muhurta-api itself calls panchang-mcp over JSON-RPC, so a 502
 * here may also mean MCP is degraded. The route handler preserves the
 * upstream status so the UI can show a precise message.
 */
export async function callMuhurta<T>(pathName: string, payload: unknown): Promise<T> {
  const base = process.env.MUHURTA_API_BASE_URL ?? DEFAULT_MUHURTA_BASE;
  return postJson<T>(base, "muhurta-api", pathName, payload);
}
