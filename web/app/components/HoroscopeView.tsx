"use client";

import { FormEvent, useMemo, useState } from "react";
import type { HoroscopeBody, HoroscopeResponse, Observer } from "@/app/lib/api";
import { formatDateLong, timeOnly } from "@/app/lib/api";

type GeocodeResponse = {
  hits: Array<{ label: string; latitude: number; longitude: number; timezone?: string | null }>;
};

type BirthPlace = {
  label: string;
  query: string;
  timezone: string;
  latitude: number;
  longitude: number;
};

type HoroscopeNotice = {
  title: string;
  body: string;
};

const GRAHA_ORDER = [
  "sun",
  "moon",
  "mars",
  "mercury",
  "jupiter",
  "venus",
  "saturn",
  "rahu",
  "ketu"
] as const;

const GRAHA_LABELS: Record<(typeof GRAHA_ORDER)[number], string> = {
  sun: "Sun",
  moon: "Moon",
  mars: "Mars",
  mercury: "Mercury",
  jupiter: "Jupiter",
  venus: "Venus",
  saturn: "Saturn",
  rahu: "Rahu",
  ketu: "Ketu"
};

async function postJson<T>(url: string, payload: unknown): Promise<T> {
  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload)
  });
  if (!res.ok) {
    const text = await res.text();
    let message = text;
    let logicalStatus = res.status;
    try {
      const j = JSON.parse(text) as {
        error?: string;
        upstreamStatus?: number;
      };
      if (typeof j.error === "string") message = j.error;
      if (typeof j.upstreamStatus === "number") logicalStatus = j.upstreamStatus;
    } catch {
      // keep raw body
    }
    throw new Error(`${logicalStatus}: ${message}`);
  }
  return (await res.json()) as T;
}

function noticeFromError(e: unknown): HoroscopeNotice {
  const raw = e instanceof Error ? e.message : "Horoscope calculation failed";
  const m = raw.match(/^(\d{3}):/);
  const status = m ? Number(m[1]) : null;
  if (status === 503) {
    return {
      title: "Horoscope MCP is unreachable",
      body: "Start it locally with: scripts/restart-api.sh horoscope-mcp"
    };
  }
  if (status === 401) {
    return {
      title: "Horoscope MCP needs credentials",
      body: "Set MCP_SHARED_SECRET on the Next.js server so it can pass the same Bearer token as horoscope-mcp."
    };
  }
  if (status === 400) {
    return { title: "Birth details need a check", body: raw.replace(/^400:\s*/, "") };
  }
  return { title: "Horoscope calculation failed", body: raw };
}

function normalizeTime(value: string): string {
  if (/^\d\d:\d\d:\d\d$/.test(value)) return value;
  if (/^\d\d:\d\d$/.test(value)) return `${value}:00`;
  return "06:00:00";
}

function formatDeg(value: number): string {
  return `${value.toFixed(2)}°`;
}

function bodyLine(body: HoroscopeBody): string {
  return `${body.rashi_name} · ${body.nakshatra_name} ${body.nakshatra_pada}`;
}

function datePart(value: string): string {
  return value.slice(0, 10);
}

function activeMahadasha(chart: HoroscopeResponse) {
  const asOf = chart.dasha_bhukti.window.as_of_local;
  return chart.dasha_bhukti.mahadashas.find((d) => d.start_local <= asOf && asOf < d.end_local) ?? null;
}

function activeAntardasha(chart: HoroscopeResponse) {
  const asOf = chart.dasha_bhukti.window.as_of_local;
  const maha = activeMahadasha(chart);
  return maha?.antardashas.find((d) => d.start_local <= asOf && asOf < d.end_local) ?? null;
}

export function HoroscopeView({
  observer,
  placeLabel,
  placeSearch
}: {
  observer: Observer;
  placeLabel: string;
  placeSearch: string;
}) {
  const [birthDate, setBirthDate] = useState("1990-01-01");
  const [birthTime, setBirthTime] = useState("06:00:00");
  const [place, setPlace] = useState<BirthPlace>({
    label: placeLabel,
    query: placeSearch || placeLabel,
    timezone: observer.timezone,
    latitude: observer.latitude,
    longitude: observer.longitude
  });
  const [dirtyPlace, setDirtyPlace] = useState(false);
  const [dashaHorizonYears, setDashaHorizonYears] = useState(20);
  const [chart, setChart] = useState<HoroscopeResponse | null>(null);
  const [notice, setNotice] = useState<HoroscopeNotice | null>(null);
  const [busy, setBusy] = useState(false);

  const activeDasha = useMemo(() => (chart ? activeMahadasha(chart) : null), [chart]);
  const activeBhukti = useMemo(() => (chart ? activeAntardasha(chart) : null), [chart]);

  async function resolvePlace(): Promise<BirthPlace> {
    if (!dirtyPlace) return place;
    const query = place.query.trim();
    if (!query) throw new Error("400: Enter a birth location.");
    const result = await postJson<GeocodeResponse>("/api/location/geocode", { query });
    const hit = result.hits[0];
    if (!hit) throw new Error("400: No address match found. Try a simpler place name.");
    const next = {
      label: hit.label,
      query,
      timezone: hit.timezone || place.timezone,
      latitude: hit.latitude,
      longitude: hit.longitude
    };
    setPlace(next);
    setDirtyPlace(false);
    return next;
  }

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setNotice(null);
    try {
      const birthPlace = await resolvePlace();
      const data = await postJson<HoroscopeResponse>("/api/horoscope/basic", {
        birth_local: `${birthDate}T${normalizeTime(birthTime)}`,
        timezone: birthPlace.timezone,
        latitude: birthPlace.latitude,
        longitude: birthPlace.longitude,
        ayanamsha: observer.ayanamsha,
        engine: observer.engine,
        dasha_horizon_years: dashaHorizonYears
      });
      setChart(data);
    } catch (err) {
      setChart(null);
      setNotice(noticeFromError(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="horoscope-view" aria-busy={busy}>
      <form className="horoscope-form" onSubmit={submit}>
        <label>
          <span className="anga-label">Birth date</span>
          <input type="date" value={birthDate} onChange={(e) => setBirthDate(e.target.value)} required />
        </label>
        <label>
          <span className="anga-label">Birth time</span>
          <input
            type="time"
            step="1"
            value={birthTime}
            onChange={(e) => setBirthTime(normalizeTime(e.target.value))}
            required
          />
        </label>
        <label className="horoscope-place-field">
          <span className="anga-label">Birth location</span>
          <input
            value={place.query}
            onChange={(e) => {
              setPlace({ ...place, query: e.target.value });
              setDirtyPlace(true);
            }}
            placeholder="City, temple, hospital, or address"
            autoComplete="off"
          />
        </label>
        <label>
          <span className="anga-label">Dasha horizon</span>
          <input
            type="number"
            min={0}
            max={80}
            value={dashaHorizonYears}
            onChange={(e) => setDashaHorizonYears(Number(e.target.value))}
          />
        </label>
        <button type="submit" className="pri horoscope-submit" disabled={busy}>
          {busy ? "Calculating..." : "Calculate"}
        </button>
      </form>

      <div className="horoscope-place-meta">
        <span>{place.label}</span>
        <span>
          {place.latitude.toFixed(4)}°, {place.longitude.toFixed(4)}° · {place.timezone}
        </span>
      </div>

      {notice ? (
        <aside className="day-detail-notice" role="alert" aria-live="polite">
          <span className="day-detail-notice-title">{notice.title}</span>
          <span className="day-detail-notice-body">{notice.body}</span>
        </aside>
      ) : null}

      {!chart && !notice ? (
        <section className="empty horoscope-empty">
          <span className="glyph" aria-hidden>
            ✧
          </span>
          <p>Enter birth details to calculate the basic South Indian natal chart.</p>
        </section>
      ) : null}

      {chart ? (
        <div className="horoscope-results">
          <article className="daily-card horoscope-summary">
            <h2>Birth chart</h2>
            <div className="anga-grid horoscope-summary-grid">
              <div className="anga-tile">
                <span className="anga-label">Lagna</span>
                <span className="anga-name">{chart.lagna.rashi_name}</span>
                <span className="anga-meta">
                  {chart.lagna.nakshatra_name} · pada {chart.lagna.nakshatra_pada}
                </span>
              </div>
              <div className="anga-tile">
                <span className="anga-label">Moon</span>
                <span className="anga-name">{chart.grahas.moon.rashi_name}</span>
                <span className="anga-meta">
                  {chart.grahas.moon.nakshatra_name} · pada {chart.grahas.moon.nakshatra_pada}
                </span>
              </div>
              <div className="anga-tile">
                <span className="anga-label">Sun</span>
                <span className="anga-name">{chart.grahas.sun.rashi_name}</span>
                <span className="anga-meta">{formatDeg(chart.grahas.sun.sidereal_longitude_deg)}</span>
              </div>
              <div className="anga-tile">
                <span className="anga-label">Tithi</span>
                <span className="anga-name">{chart.panchang_at_birth.tithi_name}</span>
                <span className="anga-meta">{chart.panchang_at_birth.paksha}</span>
              </div>
              <div className="anga-tile">
                <span className="anga-label">Current dasha</span>
                <span className="anga-name">{activeDasha?.lord_display_en ?? "—"}</span>
                <span className="anga-meta">
                  {activeBhukti ? `${activeBhukti.lord_display_en} bhukti` : "Bhukti unavailable"}
                </span>
              </div>
              <div className="anga-tile">
                <span className="anga-label">Frame</span>
                <span className="anga-name">{chart.frame.ayanamsha}</span>
                <span className="anga-meta">
                  {formatDeg(chart.frame.ayanamsha_deg)} · {chart.frame.engine}
                </span>
              </div>
            </div>
          </article>

          <div className="horoscope-grid">
            <article className="daily-card">
              <h2>Navagraha</h2>
              <ul className="graha-list">
                {GRAHA_ORDER.map((key) => {
                  const body = chart.grahas[key];
                  return (
                    <li key={key}>
                      <span className="graha-name">
                        {GRAHA_LABELS[key]}
                        {body.retrograde ? <small>R</small> : null}
                      </span>
                      <span className="graha-rashi">{bodyLine(body)}</span>
                      <span className="graha-degree">{formatDeg(body.sidereal_longitude_deg)}</span>
                    </li>
                  );
                })}
              </ul>
            </article>

            <article className="daily-card">
              <h2>Panchang at birth</h2>
              <dl className="tamil-dl">
                <dt>Vaara</dt>
                <dd>{chart.panchang_at_birth.vaara}</dd>
                <dt>Tithi</dt>
                <dd>{chart.panchang_at_birth.tithi_name}</dd>
                <dt>Yoga</dt>
                <dd>{chart.panchang_at_birth.yoga_name}</dd>
                <dt>Karana</dt>
                <dd>{chart.panchang_at_birth.karana_name}</dd>
                <dt>Tamil month</dt>
                <dd>
                  {chart.tamil_calendar_hint.solar_month_name} (
                  {chart.tamil_calendar_hint.solar_month_name_tamil})
                </dd>
                <dt>Tamil year</dt>
                <dd>{chart.tamil_calendar_hint.tamil_year_name}</dd>
              </dl>
            </article>
          </div>

          <article className="daily-card horoscope-dasha-card">
            <h2>Vimshottari dasha</h2>
            <div className="horoscope-dasha-head">
              <span>
                Moon starts in {chart.dasha_bhukti.moon_at_birth.nakshatra_name};{" "}
                {Math.round(chart.dasha_bhukti.moon_at_birth.balance_of_starting_mahadasha_at_birth_days)} days
                balance of {chart.dasha_bhukti.moon_at_birth.starting_mahadasha_lord}.
              </span>
              <span>
                As of {formatDateLong(datePart(chart.dasha_bhukti.window.as_of_local))} · through{" "}
                {formatDateLong(datePart(chart.dasha_bhukti.window.horizon_end_local))}
              </span>
            </div>
            <ol className="dasha-list">
              {chart.dasha_bhukti.mahadashas.slice(0, 8).map((dasha) => (
                <li
                  key={`${dasha.lord}-${dasha.start_local}`}
                  className={dasha === activeDasha ? "active" : ""}
                >
                  <div className="dasha-row-head">
                    <span className="dasha-lord">
                      {dasha.lord_display_en}
                      <small>{dasha.lord_display_ta}</small>
                    </span>
                    <span className="dasha-range">
                      {formatDateLong(datePart(dasha.start_local))} – {formatDateLong(datePart(dasha.end_local))}
                    </span>
                  </div>
                  {dasha === activeDasha && activeBhukti ? (
                    <p className="dasha-bhukti">
                      Current bhukti: {activeBhukti.lord_display_en} · {timeOnly(activeBhukti.start_local)}{" "}
                      {formatDateLong(datePart(activeBhukti.start_local))} –{" "}
                      {timeOnly(activeBhukti.end_local)} {formatDateLong(datePart(activeBhukti.end_local))}
                    </p>
                  ) : null}
                </li>
              ))}
            </ol>
          </article>
        </div>
      ) : null}
    </section>
  );
}
