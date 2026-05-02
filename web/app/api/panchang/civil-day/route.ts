import { NextRequest, NextResponse } from "next/server";
import { CivilDayResponse } from "@/app/lib/api";
import { callPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  const data = await callPanchang<CivilDayResponse>("/v1/panchang/civil-day", payload);
  return NextResponse.json(data);
}
