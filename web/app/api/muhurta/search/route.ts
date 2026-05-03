import { NextRequest, NextResponse } from "next/server";
import { MuhurtaResponse } from "@/app/lib/api";
import { callMuhurta, UpstreamError } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  try {
    const data = await callMuhurta<MuhurtaResponse>("/v1/muhurta/search", payload);
    return NextResponse.json(data);
  } catch (e) {
    if (e instanceof UpstreamError) {
      // Forward the muhurta-api status (incl. 502 when panchang-mcp is
      // unreachable / unauthorized) so the UI can render a precise banner.
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
    const message = e instanceof Error ? e.message : "Muhurta search failed";
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
