import { NextRequest } from "next/server";
import { CivilDayResponse } from "@/app/lib/api";
import { proxyPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  return proxyPanchang<CivilDayResponse>("/v1/panchang/civil-day", payload);
}
