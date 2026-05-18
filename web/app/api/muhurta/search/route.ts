import { NextRequest, NextResponse } from "next/server";
import { MuhurtaResponse } from "@/app/lib/api";
import { callMuhurta, upstreamErrorResponse } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  try {
    const data = await callMuhurta<MuhurtaResponse>("/v1/muhurta/search", payload);
    return NextResponse.json(data);
  } catch (e) {
    return upstreamErrorResponse(e, "Muhurta search failed");
  }
}
