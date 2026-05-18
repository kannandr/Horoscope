import "server-only";

import { NextResponse } from "next/server";

const DEFAULT_PANCHANG_BASE = "http://127.0.0.1:8080";
const DEFAULT_MUHURTA_BASE = "http://127.0.0.1:8090";
const DEFAULT_HOROSCOPE_MCP_BASE = "http://127.0.0.1:8790";

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

export function upstreamErrorResponse(e: unknown, fallbackMessage: string): NextResponse {
  if (e instanceof UpstreamError) {
    return NextResponse.json(
      {
        error: e.message,
        upstream: e.upstream,
        upstreamStatus: e.status,
        upstreamBody: e.upstreamBody
      },
      { status: e.status >= 400 && e.status < 600 ? e.status : 502 }
    );
  }
  const message = e instanceof Error ? e.message : fallbackMessage;
  return NextResponse.json({ error: message }, { status: 500 });
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

export async function proxyPanchang<T>(pathName: string, payload: unknown): Promise<NextResponse> {
  try {
    const data = await callPanchang<T>(pathName, payload);
    return NextResponse.json(data);
  } catch (e) {
    return upstreamErrorResponse(e, "Panchang request failed");
  }
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

type McpEnvelope<T> = {
  jsonrpc: "2.0";
  id?: unknown;
  result?: {
    structuredContent?: T;
    content?: Array<{ type: string; text?: string }>;
  };
  error?: {
    code: number;
    message: string;
    data?: unknown;
  };
};

/** Call the Horoscope MCP server directly over JSON-RPC (`POST /mcp`). */
export async function callHoroscopeMcpTool<T>(toolName: string, arguments_: unknown): Promise<T> {
  const base = process.env.HOROSCOPE_MCP_BASE_URL ?? DEFAULT_HOROSCOPE_MCP_BASE;
  const secret = process.env.MCP_SHARED_SECRET?.trim();
  const headers: Record<string, string> = { "content-type": "application/json" };
  if (secret) headers.authorization = `Bearer ${secret}`;

  let res: Response;
  try {
    res = await fetch(`${base}/mcp`, {
      method: "POST",
      headers,
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "tools/call",
        params: {
          name: toolName,
          arguments: arguments_
        }
      }),
      cache: "no-store"
    });
  } catch (e) {
    const cause = e instanceof Error ? e.message : String(e);
    throw new UpstreamError("horoscope-mcp", 503, `unreachable (${cause})`, null);
  }

  const text = await res.text();
  let parsed: McpEnvelope<T> | unknown = text;
  try {
    parsed = JSON.parse(text) as McpEnvelope<T>;
  } catch {
    // keep raw text for upstreamBody below
  }

  if (!res.ok) {
    const message =
      parsed && typeof parsed === "object" && "error" in parsed
        ? String((parsed as { error?: { message?: unknown } }).error?.message ?? text)
        : text;
    throw new UpstreamError("horoscope-mcp", res.status, message, parsed);
  }

  const envelope = parsed as McpEnvelope<T>;
  if (envelope.error) {
    const detail =
      envelope.error.data &&
      typeof envelope.error.data === "object" &&
      typeof (envelope.error.data as { message?: unknown }).message === "string"
        ? `: ${(envelope.error.data as { message: string }).message}`
        : "";
    throw new UpstreamError(
      "horoscope-mcp",
      envelope.error.code === -32602 ? 400 : 502,
      `${envelope.error.message}${detail}`,
      envelope.error
    );
  }

  const structured = envelope.result?.structuredContent;
  if (structured !== undefined) return structured;

  const textContent = envelope.result?.content?.find((item) => item.type === "text" && item.text)?.text;
  if (textContent) {
    try {
      return JSON.parse(textContent) as T;
    } catch {
      // fall through to explicit malformed response error
    }
  }

  throw new UpstreamError("horoscope-mcp", 502, "MCP response did not include structuredContent", parsed);
}
