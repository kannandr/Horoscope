import { NextRequest, NextResponse } from "next/server";
import { geocodeAddress } from "@/app/lib/location";

export async function POST(req: NextRequest) {
  const { query } = (await req.json()) as { query?: string };
  try {
    const hits = await geocodeAddress(typeof query === "string" ? query : "", 5);
    return NextResponse.json({ hits });
  } catch (err) {
    const message = err instanceof Error ? err.message : "Geocoding failed";
    return NextResponse.json({ hits: [], error: message }, { status: 502 });
  }
}
