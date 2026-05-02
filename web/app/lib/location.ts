import "server-only";

import tzLookup from "tz-lookup";

const NOMINATIM_BASE = process.env.NOMINATIM_BASE_URL ?? "https://nominatim.openstreetmap.org";
// OSM Nominatim usage policy: identify the application via User-Agent.
const NOMINATIM_UA =
  process.env.NOMINATIM_USER_AGENT ??
  "PanchangCalendar/1.0 (https://github.com/example/horoscope-panchang; educational use)";
const NOMINATIM_TIMEOUT_MS = 12_000;

export type GeocodeHit = {
  label: string;
  latitude: number;
  longitude: number;
  timezone: string | null;
};

export type ReverseResult = {
  label: string;
  search_query: string;
  latitude: number;
  longitude: number;
  timezone: string | null;
};

type NominatimSearchRow = {
  lat?: string;
  lon?: string;
  display_name?: string;
};

type NominatimReverseAddress = {
  city?: string;
  town?: string;
  village?: string;
  hamlet?: string;
  suburb?: string;
  neighbourhood?: string;
  municipality?: string;
  county?: string;
  state?: string;
  region?: string;
  country?: string;
};

type NominatimReverseRow = {
  display_name?: string;
  address?: NominatimReverseAddress;
};

async function fetchJson<T>(url: string): Promise<T> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), NOMINATIM_TIMEOUT_MS);
  try {
    const res = await fetch(url, {
      headers: {
        "User-Agent": NOMINATIM_UA,
        accept: "application/json"
      },
      signal: controller.signal,
      cache: "no-store"
    });
    if (!res.ok) {
      throw new Error(`Nominatim ${res.status}`);
    }
    return (await res.json()) as T;
  } finally {
    clearTimeout(timer);
  }
}

function timezoneAt(latitude: number, longitude: number): string | null {
  try {
    return tzLookup(latitude, longitude);
  } catch {
    return null;
  }
}

export async function geocodeAddress(query: string, limit = 5): Promise<GeocodeHit[]> {
  const q = query.trim();
  if (!q) return [];
  const url = `${NOMINATIM_BASE}/search?format=json&addressdetails=1&limit=${limit}&q=${encodeURIComponent(q)}`;
  const rows = await fetchJson<NominatimSearchRow[]>(url);
  const hits: GeocodeHit[] = [];
  for (const row of rows) {
    if (!row.lat || !row.lon) continue;
    const latitude = Number(row.lat);
    const longitude = Number(row.lon);
    if (!Number.isFinite(latitude) || !Number.isFinite(longitude)) continue;
    hits.push({
      label: row.display_name ?? `${latitude}, ${longitude}`,
      latitude,
      longitude,
      timezone: timezoneAt(latitude, longitude)
    });
  }
  return hits;
}

function searchQueryFromAddress(addr: NominatimReverseAddress, latitude: number, longitude: number): string {
  const place =
    addr.city ??
    addr.town ??
    addr.village ??
    addr.hamlet ??
    addr.suburb ??
    addr.neighbourhood ??
    addr.municipality ??
    addr.county ??
    null;
  const state = addr.state ?? addr.region ?? null;
  const country = addr.country ?? null;
  const bits = [place, state, country].filter((x): x is string => Boolean(x));
  if (bits.length >= 2) return bits.join(", ");
  if (place) return place;
  if (state && country) return `${state}, ${country}`;
  return `${latitude.toFixed(5)}, ${longitude.toFixed(5)}`;
}

export async function reverseGeocode(latitude: number, longitude: number): Promise<ReverseResult> {
  const fallback: ReverseResult = {
    label: `${latitude.toFixed(6)}, ${longitude.toFixed(6)}`,
    search_query: `${latitude.toFixed(5)}, ${longitude.toFixed(5)}`,
    latitude,
    longitude,
    timezone: timezoneAt(latitude, longitude)
  };
  try {
    const url = `${NOMINATIM_BASE}/reverse?format=json&addressdetails=1&lat=${encodeURIComponent(
      latitude.toString()
    )}&lon=${encodeURIComponent(longitude.toString())}`;
    const row = await fetchJson<NominatimReverseRow>(url);
    const label = (row.display_name ?? "").trim() || fallback.label;
    const addr = row.address ?? {};
    return {
      ...fallback,
      label,
      search_query: searchQueryFromAddress(addr, latitude, longitude)
    };
  } catch {
    return fallback;
  }
}
