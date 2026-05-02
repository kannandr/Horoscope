import { NextRequest, NextResponse } from "next/server";
import { PanchangDayResponse } from "@/app/lib/api";
import { callPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  const data = await callPanchang<PanchangDayResponse>("/v1/panchang/day", payload);
  return NextResponse.json(data);
}
