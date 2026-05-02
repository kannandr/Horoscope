import { NextRequest, NextResponse } from "next/server";
import { MuhurtaResponse } from "@/app/lib/api";
import { callPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  const data = await callPanchang<MuhurtaResponse>("/v1/muhurta/search", payload);
  return NextResponse.json(data);
}
