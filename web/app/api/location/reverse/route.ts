import { NextRequest, NextResponse } from "next/server";
import { reverseGeocode } from "@/app/lib/location";

export async function POST(req: NextRequest) {
  const body = (await req.json()) as { latitude?: unknown; longitude?: unknown };
  const latitude = Number(body.latitude);
  const longitude = Number(body.longitude);
  if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) {
    return NextResponse.json({ error: "Invalid coordinates" }, { status: 400 });
  }
  const result = await reverseGeocode(latitude, longitude);
  return NextResponse.json(result);
}
