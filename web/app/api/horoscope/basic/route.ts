import { NextRequest, NextResponse } from "next/server";
import type { HoroscopeResponse } from "@/app/lib/api";
import { callHoroscopeMcpTool, upstreamErrorResponse } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  try {
    const data = await callHoroscopeMcpTool<HoroscopeResponse>(
      "calculate_south_indian_natal_chart",
      payload
    );
    return NextResponse.json(data);
  } catch (e) {
    return upstreamErrorResponse(e, "Horoscope calculation failed");
  }
}
