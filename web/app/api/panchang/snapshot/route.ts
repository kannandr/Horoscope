import { NextRequest, NextResponse } from "next/server";
import { SnapshotResponse } from "@/app/lib/api";
import { callPanchang } from "@/app/lib/serverApi";

export async function POST(req: NextRequest) {
  const payload = await req.json();
  const data = await callPanchang<SnapshotResponse>("/v1/panchang/snapshot", payload);
  return NextResponse.json(data);
}
