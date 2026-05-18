import { NextRequest } from "next/server";
import { MonthResponse } from "@/app/lib/api";
import { proxyPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  return proxyPanchang<MonthResponse>("/v1/panchang/month", payload);
}
