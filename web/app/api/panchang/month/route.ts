import { NextRequest, NextResponse } from "next/server";
import { MonthResponse } from "@/app/lib/api";
import { callPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  const data = await callPanchang<MonthResponse>("/v1/panchang/month", payload);
  return NextResponse.json(data);
}
