import "server-only";

const DEFAULT_API_BASE = "http://localhost:8080";

export async function callPanchang<T>(pathName: string, payload: unknown): Promise<T> {
  const base = process.env.PANCHANG_API_BASE_URL ?? DEFAULT_API_BASE;
  const res = await fetch(`${base}${pathName}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
    cache: "no-store"
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Panchang API ${res.status}: ${body}`);
  }
  return (await res.json()) as T;
}
