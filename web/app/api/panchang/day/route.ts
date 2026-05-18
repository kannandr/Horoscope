import { NextRequest } from "next/server";
import { PanchangDayResponse } from "@/app/lib/api";
import { proxyPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  return proxyPanchang<PanchangDayResponse>("/v1/panchang/day", payload);
}
