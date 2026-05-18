import { NextRequest } from "next/server";
import { SnapshotResponse } from "@/app/lib/api";
import { proxyPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  return proxyPanchang<SnapshotResponse>("/v1/panchang/snapshot", payload);
}
