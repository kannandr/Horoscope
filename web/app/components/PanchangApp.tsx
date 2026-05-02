"use client";

import { FormEvent, useEffect, useMemo, useRef, useState } from "react";
import type {
  CivilDayResponse,
  MonthResponse,
  MuhurtaResponse,
  Observer,
  PanchangDayResponse,
  SnapshotResponse
} from "@/app/lib/api";
import { timeOnly } from "@/app/lib/api";
import { AuspiciousView } from "@/app/components/AuspiciousView";
import { parseMuhurtaQuery } from "@/app/lib/muhurtaParse";
import {
  panchangDayFromCivilAndSnapshot,
  panchangDayFromSnapshotOnly
} from "@/app/lib/panchangDayFallback";

type View = "month" | "week" | "day" | "auspicious";

/** Short banner when full-day API is unavailable (still all-local Rust). */
type DayDetailNotice = { title: string; body: string };

type GeocodeResponse = {
  hits: Array<{ label: string; latitude: number; longitude: number; timezone?: string | null }>;
};

type ReverseResponse = {
  label: string;
  search_query: string;
  latitude: number;
  longitude: number;
  timezone?: string | null;
};

type AngaKind = "tithi" | "nakshatra";

type TransitionEvent = {
  name: string;
  kind: AngaKind;
  action: "starts" | "ends";
  timeLocal: string;
  pada?: number | null;
};

const defaultObserver: Observer = {
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "America/Los_Angeles",
  latitude: 37.6819,
  longitude: -121.768,
  ayanamsha: "lahiri",
  engine: "meeus"
};

/* ============================================================
   Date helpers
   We never call toISOString() on a Date built from local time,
   because that silently converts to UTC and drifts by ±1 day
   near midnight. All date arithmetic happens on YYYY-MM-DD
   strings via Date.UTC(...) math.
   "Today" is always computed in a specific IANA timezone.
   ============================================================ */

function isoDateInTz(when: Date, timeZone: string): string {
  // en-CA gives YYYY-MM-DD ordering deterministically.
  const fmt = new Intl.DateTimeFormat("en-CA", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit"
  });
  return fmt.format(when);
}

function todayIsoInTz(timeZone: string): string {
  return isoDateInTz(new Date(), timeZone);
}

/* Wall-clock HH:MM:SS in a specific IANA timezone. Used to seed and tick the
   daily snapshot so the day view defaults to "right now" instead of noon. */
function nowTimeInTz(timeZone: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    timeZone,
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit"
  }).format(new Date());
}

function parseIso(iso: string): { y: number; m: number; d: number } {
  const [y, m, d] = iso.split("-").map(Number);
  return { y, m, d };
}

function fmtIsoUtc(dt: Date): string {
  const y = dt.getUTCFullYear();
  const m = String(dt.getUTCMonth() + 1).padStart(2, "0");
  const d = String(dt.getUTCDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function addDaysIso(iso: string, days: number): string {
  const { y, m, d } = parseIso(iso);
  const dt = new Date(Date.UTC(y, m - 1, d));
  dt.setUTCDate(dt.getUTCDate() + days);
  return fmtIsoUtc(dt);
}

function addMonthsIso(iso: string, months: number): string {
  const { y, m } = parseIso(iso);
  const dt = new Date(Date.UTC(y, m - 1 + months, 1));
  return fmtIsoUtc(dt);
}

function isoWeekday(iso: string): number {
  // Returns 0..6 with Monday = 0.
  const { y, m, d } = parseIso(iso);
  const wd = new Date(Date.UTC(y, m - 1, d)).getUTCDay(); // Sunday = 0
  return (wd + 6) % 7;
}

function isoWeekStart(iso: string): string {
  return addDaysIso(iso, -isoWeekday(iso));
}

function isoMonth(iso: string): { year: number; month: number } {
  const { y, m } = parseIso(iso);
  return { year: y, month: m };
}

function sameMonth(a: { year: number; month: number }, b: { year: number; month: number }): boolean {
  return a.year === b.year && a.month === b.month;
}

const WEEKDAY_HEADERS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;
const MAX_TX_PER_CELL = 5;
const MAX_TX_PER_WEEK_CARD = 14;

const TITHI_GLYPH = "◐";
const PURNIMA_GLYPH = "●";
const AMAVASYA_GLYPH = "○";
const NAKSHATRA_GLYPH = "✦";

function tithiGlyph(name?: string | null): string {
  if (!name) return TITHI_GLYPH;
  const lower = name.toLowerCase();
  if (lower.includes("purnima")) return PURNIMA_GLYPH;
  if (lower.includes("amavasya")) return AMAVASYA_GLYPH;
  return TITHI_GLYPH;
}

/** Convert a Julian Date (UT) to a HH:MM:SS string in the given IANA timezone.
 * JD epoch is 12:00 UT on January 1, 4713 BC (proleptic Julian); the Unix epoch
 * is JD 2440587.5. */
function jdToLocalTime(jd: number | null | undefined, timeZone: string): string | null {
  if (jd == null || !Number.isFinite(jd)) return null;
  const unixMs = (jd - 2440587.5) * 86_400_000;
  const dt = new Date(unixMs);
  return new Intl.DateTimeFormat("en-GB", {
    timeZone,
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit"
  }).format(dt);
}

async function postJson<T>(url: string, payload: unknown): Promise<T> {
  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload)
  });
  if (!res.ok) {
    throw new Error(await res.text());
  }
  return (await res.json()) as T;
}

export default function PanchangApp() {
  const [observer, setObserver] = useState<Observer>(defaultObserver);
  const todayIso = useMemo(() => todayIsoInTz(observer.timezone), [observer.timezone]);
  const initialToday = useMemo(() => todayIsoInTz(defaultObserver.timezone), []);
  const [date, setDate] = useState(initialToday);
  /* Lazy-init the snapshot time to the current wall clock in the default
     observer's timezone, so opening the Day view shows live data instead of
     a stale "12:00:00". A `liveTime` flag below keeps that fresh on a
     30-second tick whenever we're viewing today's day-view; manually editing
     the time input or navigating away pauses the auto-update. */
  const [time, setTime] = useState<string>(() => nowTimeInTz(defaultObserver.timezone));
  const [liveTime, setLiveTime] = useState(true);
  const [address, setAddress] = useState("");
  const [placeLabel, setPlaceLabel] = useState("Livermore, California, United States");
  const [view, setView] = useState<View>("month");
  const [month, setMonth] = useState<MonthResponse | null>(null);
  const [panchangDay, setPanchangDay] = useState<PanchangDayResponse | null>(null);
  const [snapshot, setSnapshot] = useState<SnapshotResponse | null>(null);
  const [muhurta, setMuhurta] = useState<MuhurtaResponse | null>(null);
  const [muhurtaQuery, setMuhurtaQuery] = useState(
    "Find auspicious daytime windows this week for an important event. Prefer at least 45 minutes."
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dayDetailNotice, setDayDetailNotice] = useState<DayDetailNotice | null>(null);
  const didAutoLoad = useRef(false);

  async function fetchMonth(year: number, month_: number, obs: Observer): Promise<MonthResponse> {
    return postJson<MonthResponse>("/api/panchang/month", { year, month: month_, ...obs });
  }

  async function compute(
    nextView: View = view,
    nextDate: string = date,
    nextObserver: Observer = observer,
    nextTime: string = time
  ) {
    setBusy(true);
    setError(null);
    setDayDetailNotice(null);
    try {
      if (nextView === "month") {
        const m = isoMonth(nextDate);
        const data = await fetchMonth(m.year, m.month, nextObserver);
        setMonth(data);
      } else if (nextView === "week") {
        const start = isoWeekStart(nextDate);
        const end = addDaysIso(start, 6);
        const startMonth = isoMonth(start);
        const endMonth = isoMonth(end);
        if (sameMonth(startMonth, endMonth)) {
          const data = await fetchMonth(startMonth.year, startMonth.month, nextObserver);
          setMonth(data);
        } else {
          // Week straddles two months — fetch both so the strip is never half-empty.
          const [a, b] = await Promise.all([
            fetchMonth(startMonth.year, startMonth.month, nextObserver),
            fetchMonth(endMonth.year, endMonth.month, nextObserver)
          ]);
          // Anchor metadata to the month containing nextDate.
          const anchor = isoMonth(nextDate);
          const merged: MonthResponse = {
            year: anchor.year,
            month: anchor.month,
            timezone: a.timezone,
            days: [...a.days, ...b.days]
          };
          setMonth(merged);
        }
      } else if (nextView === "day") {
        const whenLocal = `${nextDate}T${nextTime}`;
        const snap = await postJson<SnapshotResponse>("/api/panchang/snapshot", {
          when_local: whenLocal,
          ...nextObserver
        });
        let pd: PanchangDayResponse;
        let detailNotice: DayDetailNotice | null = null;
        try {
          pd = await postJson<PanchangDayResponse>("/api/panchang/day", {
            date: nextDate,
            day_mode: "civil_midnight",
            ...nextObserver
          });
        } catch (dayErr) {
          const dayMsg = dayErr instanceof Error ? dayErr.message : "Day request failed";
          const looksLikeStaleBinary = /\b404\b/.test(dayMsg);
          try {
            const civil = await postJson<CivilDayResponse>("/api/panchang/civil-day", {
              date: nextDate,
              ...nextObserver
            });
            pd = panchangDayFromCivilAndSnapshot(civil, snap);
            detailNotice = looksLikeStaleBinary
              ? {
                  title: "Local Panchang server is out of date",
                  body: "Tithi and Nakshatra ranges are showing, but Tamil calendar and the day caution windows need a newer build of the local engine. Restart it with: cd rust && cargo run -p panchang-api --release"
                }
              : {
                  title: "Partial day detail",
                  body: "Tithi and Nakshatra ranges are showing. Tamil calendar and the day caution windows did not load this round; try Refresh in a few seconds."
                };
          } catch {
            pd = panchangDayFromSnapshotOnly(nextDate, nextObserver.timezone, snap);
            detailNotice = {
              title: "Local Panchang server is unreachable",
              body: `Showing only the snapshot Panchang for the time picker. Start the local engine with:  cd rust && cargo run -p panchang-api --release   (raw error: ${dayMsg})`
            };
          }
        }
        setSnapshot(snap);
        setPanchangDay(pd);
        setDayDetailNotice(detailNotice);
      } else if (nextView === "auspicious") {
        const parsed = parseMuhurtaQuery(muhurtaQuery, nextDate);
        const data = await postJson<MuhurtaResponse>("/api/muhurta/search", {
          date_start: parsed.date_start,
          date_end: parsed.date_end,
          timezone: nextObserver.timezone,
          latitude: nextObserver.latitude,
          longitude: nextObserver.longitude,
          purpose_preset: parsed.purpose_preset,
          min_duration_minutes: parsed.min_duration_minutes,
          ayanamsha: nextObserver.ayanamsha,
          engine: nextObserver.engine
        });
        setMuhurta(data);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Calculation failed");
    } finally {
      setBusy(false);
    }
  }

  function goPrev() {
    let next = date;
    if (view === "month") next = addMonthsIso(date, -1);
    else if (view === "week") next = addDaysIso(date, -7);
    else if (view === "auspicious") next = addDaysIso(date, -7);
    else next = addDaysIso(date, -1);
    setDate(next);
    void compute(view, next);
  }

  function goNext() {
    let next = date;
    if (view === "month") next = addMonthsIso(date, 1);
    else if (view === "week") next = addDaysIso(date, 7);
    else if (view === "auspicious") next = addDaysIso(date, 7);
    else next = addDaysIso(date, 1);
    setDate(next);
    void compute(view, next);
  }

  function goToday() {
    const t = todayIsoInTz(observer.timezone);
    const nowT = nowTimeInTz(observer.timezone);
    setDate(t);
    setTime(nowT);
    setLiveTime(true);
    void compute(view, t, observer, nowT);
  }

  /* Re-engage live mode: snap time to "now" and trigger a recompute. Called
     from the "Now" button in the daily nav. */
  function syncNow() {
    const nowT = nowTimeInTz(observer.timezone);
    setTime(nowT);
    setLiveTime(true);
    if (view === "day") void compute("day", date, observer, nowT);
  }

  async function runMuhurtaSearch(queryText: string) {
    setMuhurtaQuery(queryText);
    setBusy(true);
    setError(null);
    try {
      const parsed = parseMuhurtaQuery(queryText, date);
      const data = await postJson<MuhurtaResponse>("/api/muhurta/search", {
        date_start: parsed.date_start,
        date_end: parsed.date_end,
        timezone: observer.timezone,
        latitude: observer.latitude,
        longitude: observer.longitude,
        purpose_preset: parsed.purpose_preset,
        min_duration_minutes: parsed.min_duration_minutes,
        ayanamsha: observer.ayanamsha,
        engine: observer.engine
      });
      setMuhurta(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Muhurta search failed");
    } finally {
      setBusy(false);
    }
  }

  /* Switch the active view. When entering the day view in live mode we
     also snap the snapshot time to "now" in the observer's timezone so the
     first fetch reflects the real wall clock instead of whatever stale time
     was last set. */
  function switchView(v: View) {
    setView(v);
    if (v === "day" && liveTime) {
      const nowT = nowTimeInTz(observer.timezone);
      setTime(nowT);
      void compute(v, date, observer, nowT);
    } else {
      void compute(v);
    }
  }

  /* Keyboard shortcuts: M/W/D/A for views, T for today, ←/→ for prev/next, / to focus search.
     Inputs and contenteditable nodes are skipped so users can type normally. */
  const goPrevRef = useRef(goPrev);
  const goNextRef = useRef(goNext);
  const goTodayRef = useRef(goToday);
  const switchViewRef = useRef(switchView);
  goPrevRef.current = goPrev;
  goNextRef.current = goNext;
  goTodayRef.current = goToday;
  switchViewRef.current = switchView;
  useEffect(() => {
    function isTypingTarget(target: EventTarget | null): boolean {
      const el = target as HTMLElement | null;
      if (!el) return false;
      const tag = el.tagName;
      return (
        tag === "INPUT" ||
        tag === "TEXTAREA" ||
        tag === "SELECT" ||
        el.isContentEditable === true
      );
    }
    function onKey(e: KeyboardEvent) {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      if (e.key === "/" && !isTypingTarget(e.target)) {
        const el = document.querySelector<HTMLInputElement>(".locator input");
        if (el) {
          e.preventDefault();
          el.focus();
          el.select();
        }
        return;
      }
      if (isTypingTarget(e.target)) return;
      const k = e.key.toLowerCase();
      if (k === "m") {
        switchViewRef.current("month");
      } else if (k === "w") {
        switchViewRef.current("week");
      } else if (k === "d") {
        switchViewRef.current("day");
      } else if (k === "a") {
        switchViewRef.current("auspicious");
      } else if (k === "t") {
        goTodayRef.current();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        goPrevRef.current();
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        goNextRef.current();
      }
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (didAutoLoad.current) return;
    didAutoLoad.current = true;
    void compute("month");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  /* Live-tick the daily snapshot. While the user is on the day view AND
     viewing today AND hasn't manually pinned the time, we re-pull a fresh
     snapshot every 30 seconds so Hora-now / current Tithi / etc reflect the
     real current moment without the user clicking Refresh. The interval
     auto-pauses whenever any condition changes (view, location, day,
     manual time edit). */
  useEffect(() => {
    if (view !== "day") return;
    if (!liveTime) return;
    if (date !== todayIso) return;
    const tick = () => {
      const t = nowTimeInTz(observer.timezone);
      setTime(t);
      void compute("day", date, observer, t);
    };
    const id = window.setInterval(tick, 30_000);
    return () => window.clearInterval(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view, liveTime, date, todayIso, observer.timezone, observer.latitude, observer.longitude]);

  function submit(e: FormEvent) {
    e.preventDefault();
    void searchAddress();
  }

  async function applyObserverAndRecompute(nextObserver: Observer) {
    setObserver(nextObserver);
    /* When live mode is on, the snapshot time should follow the new
       timezone's wall clock; otherwise keep whatever time the user pinned. */
    const nextT = liveTime ? nowTimeInTz(nextObserver.timezone) : time;
    if (liveTime) setTime(nextT);
    try {
      await compute(view, date, nextObserver, nextT);
    } catch (e) {
      setError(
        e instanceof Error
          ? `Location updated, but calculation failed: ${e.message}`
          : "Location updated, but calculation failed."
      );
    }
  }

  function useBrowserLocation() {
    if (!("geolocation" in navigator)) {
      setError("Geolocation is not supported in this browser.");
      return;
    }
    setBusy(true);
    setError(null);
    navigator.geolocation.getCurrentPosition(
      async (pos) => {
        const latitude = Number(pos.coords.latitude.toFixed(6));
        const longitude = Number(pos.coords.longitude.toFixed(6));
        let nextObserver: Observer = { ...observer, latitude, longitude };
        try {
          const result = await postJson<ReverseResponse>("/api/location/reverse", { latitude, longitude });
          setPlaceLabel(result.label);
          setAddress(result.search_query);
          nextObserver = { ...nextObserver, timezone: result.timezone || observer.timezone };
        } catch (e) {
          setError(
            e instanceof Error
              ? `Reverse geocode failed (${e.message}); using raw coordinates.`
              : "Reverse geocode failed; using raw coordinates."
          );
          setPlaceLabel(`${latitude.toFixed(5)}, ${longitude.toFixed(5)}`);
          setAddress(`${latitude.toFixed(5)}, ${longitude.toFixed(5)}`);
        }
        await applyObserverAndRecompute(nextObserver);
        setBusy(false);
      },
      (err) => {
        setBusy(false);
        setError(err.message);
      },
      { enableHighAccuracy: true, timeout: 10000 }
    );
  }

  async function searchAddress() {
    if (!address.trim()) {
      void compute();
      return;
    }
    setBusy(true);
    setError(null);
    let result: GeocodeResponse;
    try {
      result = await postJson<GeocodeResponse>("/api/location/geocode", { query: address.trim() });
    } catch (e) {
      setError(e instanceof Error ? `Address lookup failed: ${e.message}` : "Address lookup failed.");
      setBusy(false);
      return;
    }
    const hit = result.hits[0];
    if (!hit) {
      setError("No address match found. Try a simpler place name.");
      setBusy(false);
      return;
    }
    setPlaceLabel(hit.label);
    const nextObserver: Observer = {
      ...observer,
      latitude: hit.latitude,
      longitude: hit.longitude,
      timezone: hit.timezone || observer.timezone
    };
    await applyObserverAndRecompute(nextObserver);
    setBusy(false);
  }

  const monthIndex = useMemo(() => {
    const out = new Map<string, MonthResponse["days"][number]>();
    if (month) {
      for (const day of month.days) out.set(day.date, day);
    }
    return out;
  }, [month]);

  const weekDays = useMemo(() => {
    if (!month) return [] as Array<MonthResponse["days"][number] | null>;
    const monday = isoWeekStart(date);
    return Array.from({ length: 7 }, (_, i) => {
      const iso = addDaysIso(monday, i);
      return monthIndex.get(iso) ?? null;
    });
  }, [date, month, monthIndex]);

  /** Format a YYYY-MM-DD as a Date pinned to UTC noon — safe for `toLocaleString`
   * because we then read year/month/day-of-month from the same instant; we never
   * cross a date boundary because we're only formatting calendar fields. */
  function dateForDisplay(iso: string): Date {
    const { y, m, d } = parseIso(iso);
    return new Date(Date.UTC(y, m - 1, d, 12, 0, 0));
  }

  const heading = useMemo(() => {
    const fmt = (iso: string, opts: Intl.DateTimeFormatOptions) =>
      dateForDisplay(iso).toLocaleString(undefined, { ...opts, timeZone: "UTC" });
    if (view === "auspicious") {
      return {
        title: "Auspicious times",
        sub: "Tamil / South Indian preset · describe your event below"
      };
    }
    if (view === "day") {
      return {
        title: fmt(date, { weekday: "long", day: "numeric", month: "long", year: "numeric" }),
        sub: "Daily Panchang"
      };
    }
    if (view === "week") {
      const monday = isoWeekStart(date);
      const sunday = addDaysIso(monday, 6);
      const startMonth = isoMonth(monday);
      const endMonth = isoMonth(sunday);
      const span = sameMonth(startMonth, endMonth)
        ? `${fmt(monday, { day: "numeric" })} – ${fmt(sunday, { day: "numeric", month: "long", year: "numeric" })}`
        : `${fmt(monday, { day: "numeric", month: "short" })} – ${fmt(sunday, { day: "numeric", month: "short", year: "numeric" })}`;
      return { title: span, sub: "Week" };
    }
    return {
      title: fmt(date, { month: "long", year: "numeric" }),
      sub: "Month"
    };
  }, [date, view]);

  const isOnToday = date === todayIso;

  return (
    <main className="app-shell">
      <header className="appbar">
        <div className="appbar-inner">
          <div className="brand-stack">
            <span className="brand">Panchang</span>
            <span className="brand-sub">Vedic Almanac</span>
          </div>
          <form className="locator" onSubmit={submit} role="search" aria-label="Location search">
            <span className="locator-glyph" aria-hidden>⌕</span>
            <input
              value={address}
              placeholder="Search city, temple, or address"
              onChange={(e) => setAddress(e.target.value)}
              aria-label="Location"
              autoComplete="off"
              spellCheck={false}
            />
            <button
              type="button"
              className="locator-icon-btn"
              title="Use current location"
              aria-label="Use current location"
              onClick={useBrowserLocation}
              disabled={busy}
            >
              ⌖
            </button>
            <button type="submit" className="pri" disabled={busy}>
              {busy ? "…" : "Search"}
            </button>
          </form>
          <div className="appbar-actions">
            <div className="place-stack" title={`${placeLabel} · ${observer.timezone}`}>
              <span className="place-name">{placeLabel}</span>
              <span className="place-meta">
                <LocalClock timezone={observer.timezone} />
                <span className="place-tz">{observer.timezone}</span>
              </span>
              <span className="place-coords">
                {observer.latitude.toFixed(3)}°, {observer.longitude.toFixed(3)}°
              </span>
            </div>
            <ThemeToggle />
          </div>
        </div>
        <div className="advanced-bar">
          <details className="advanced">
            <summary>Calculation settings</summary>
            <div className="grid">
              <label>
                Timezone
                <input
                  value={observer.timezone}
                  onChange={(e) => setObserver({ ...observer, timezone: e.target.value })}
                />
              </label>
              <label>
                Latitude
                <input
                  type="number"
                  step="0.000001"
                  value={observer.latitude}
                  onChange={(e) => setObserver({ ...observer, latitude: Number(e.target.value) })}
                />
              </label>
              <label>
                Longitude
                <input
                  type="number"
                  step="0.000001"
                  value={observer.longitude}
                  onChange={(e) => setObserver({ ...observer, longitude: Number(e.target.value) })}
                />
              </label>
              <label>
                Engine
                <select
                  value={observer.engine}
                  onChange={(e) => setObserver({ ...observer, engine: e.target.value as Observer["engine"] })}
                >
                  <option value="meeus">Meeus</option>
                  <option value="surya_mean">Surya mean</option>
                </select>
              </label>
              <label>
                Ayanamsha
                <select
                  value={observer.ayanamsha}
                  onChange={(e) =>
                    setObserver({ ...observer, ayanamsha: e.target.value as Observer["ayanamsha"] })
                  }
                >
                  <option value="lahiri">Lahiri</option>
                  <option value="lahiri_alt_stub">Lahiri alt stub</option>
                  <option value="raman">Raman</option>
                </select>
              </label>
            </div>
          </details>
        </div>
      </header>

      <div className="app-main">
      <section className="subbar">
        <div className="subbar-nav" aria-label="Date navigation">
          <button
            type="button"
            className="nav-btn"
            onClick={goPrev}
            title={
              view === "month"
                ? "Previous month"
                : view === "week"
                  ? "Previous week"
                  : view === "auspicious"
                    ? "Shift window earlier"
                    : "Previous day"
            }
            aria-label={
              view === "month"
                ? "Previous month"
                : view === "week"
                  ? "Previous week"
                  : view === "auspicious"
                    ? "Shift search window earlier"
                    : "Previous day"
            }
          >
            ‹
          </button>
          <button
            type="button"
            className={`nav-btn today-btn${isOnToday ? " active" : ""}`}
            onClick={goToday}
            title="Jump to today"
          >
            Today
          </button>
          <button
            type="button"
            className="nav-btn"
            onClick={goNext}
            title={
              view === "month"
                ? "Next month"
                : view === "week"
                  ? "Next week"
                  : view === "auspicious"
                    ? "Shift window later"
                    : "Next day"
            }
            aria-label={
              view === "month"
                ? "Next month"
                : view === "week"
                  ? "Next week"
                  : view === "auspicious"
                    ? "Shift search window later"
                    : "Next day"
            }
          >
            ›
          </button>
        </div>
        <h1 className="month-label">
          <span>{heading.title}</span>
          <span className="month-sub">{heading.sub}</span>
        </h1>
        <nav className="seg" role="tablist" aria-label="Calendar views">
          {(["month", "week", "day", "auspicious"] as View[]).map((v) => (
            <button
              key={v}
              role="tab"
              aria-selected={view === v}
              tabIndex={view === v ? 0 : -1}
              onClick={() => switchView(v)}
            >
              {v === "auspicious" ? "Auspicious" : v[0].toUpperCase() + v.slice(1)}
            </button>
          ))}
        </nav>
      </section>

      {error && (
        <div className="error" role="alert">
          {error}
        </div>
      )}

      {(view === "month" || view === "week") && <Legend />}
      <div key={view} className="view-fade view-area">
        {view === "month" && (
          <MonthGrid month={month} anchorDate={date} todayIso={todayIso} busy={busy} />
        )}
        {view === "week" && <WeekStrip days={weekDays} todayIso={todayIso} busy={busy} />}
        {view === "day" && (
          <DailyView
            panchangDay={panchangDay}
            snapshot={snapshot}
            dayDetailNotice={dayDetailNotice}
            dayDetailDegraded={dayDetailNotice !== null}
            date={date}
            time={time}
            timezone={observer.timezone}
            live={liveTime && date === todayIso}
            onDateChange={(nextDate) => {
              setDate(nextDate);
              void compute("day", nextDate);
            }}
            onTimeChange={(t) => {
              /* Manually scrubbing the time freezes auto-update so the user
                 stays in control; "Now" puts us back into live mode. */
              setTime(t);
              setLiveTime(false);
              void compute("day", date, observer, t);
            }}
            onSyncNow={syncNow}
            onPrevious={goPrev}
            onNext={goNext}
            onCompute={() => void compute("day")}
            busy={busy}
          />
        )}
        {view === "auspicious" && (
          <AuspiciousView
            anchorDate={date}
            observer={observer}
            result={muhurta}
            busy={busy}
            onSearch={runMuhurtaSearch}
            onAnchorDateChange={(iso) => {
              setDate(iso);
              void compute("auspicious", iso);
            }}
          />
        )}
      </div>
      </div>
    </main>
  );
}

/* ============================================================
   Live local clock — instrument-grade detail in the appbar.
   Updates once per second, formatted in observer.timezone.
   Hidden until first effect runs to avoid hydration mismatch.
   ============================================================ */
function LocalClock({ timezone }: { timezone: string }) {
  const [now, setNow] = useState<Date | null>(null);
  useEffect(() => {
    setNow(new Date());
    const id = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(id);
  }, []);
  const fmt = useMemo(
    () =>
      new Intl.DateTimeFormat("en-GB", {
        timeZone: timezone,
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit"
      }),
    [timezone]
  );
  if (!now) return <span className="local-clock" aria-hidden suppressHydrationWarning>--:--:--</span>;
  return (
    <span className="local-clock" aria-label={`Current time at observer location in ${timezone}`}>
      {fmt.format(now)}
    </span>
  );
}

/* ============================================================
   Theme toggle — light / dark.
   The actual data-theme attribute is set pre-hydration by an inline
   script in layout.tsx to prevent a flash; this component just
   reflects + toggles it.
   ============================================================ */
function ThemeToggle() {
  const [theme, setTheme] = useState<"light" | "dark" | null>(null);
  useEffect(() => {
    const initial =
      (document.documentElement.dataset.theme as "light" | "dark" | undefined) ?? "light";
    setTheme(initial);
  }, []);
  function toggle() {
    const next = theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    try {
      localStorage.setItem("theme", next);
    } catch {
      // localStorage may be blocked (Safari private mode, etc.) — non-fatal.
    }
    setTheme(next);
  }
  if (!theme) {
    // Render placeholder with the same dimensions to avoid layout shift.
    return <span className="theme-toggle" aria-hidden style={{ visibility: "hidden" }} />;
  }
  const isDark = theme === "dark";
  return (
    <button
      type="button"
      className="theme-toggle"
      onClick={toggle}
      title={`Switch to ${isDark ? "light" : "dark"} mode`}
      aria-label={`Switch to ${isDark ? "light" : "dark"} mode`}
    >
      {isDark ? "☀" : "☾"}
    </button>
  );
}

function Legend() {
  return (
    <div className="tx-legend" aria-hidden>
      <span className="legend-item">
        <span className="legend-pill tithi">{TITHI_GLYPH} Tithi</span>
      </span>
      <span className="legend-item">
        <span className="legend-pill nakshatra">{NAKSHATRA_GLYPH} Nakshatra</span>
      </span>
      <span className="sep" />
      <span className="legend-item">
        <span className="marker start" /> starts
      </span>
      <span className="legend-item">
        <span className="marker end" /> ends
      </span>
    </div>
  );
}

function MonthGrid({
  month,
  anchorDate,
  todayIso,
  busy
}: {
  month: MonthResponse | null;
  anchorDate: string;
  todayIso: string;
  busy: boolean;
}) {
  /* When the underlying month payload was merged from two months (week-straddle),
     anchor metadata reflects the month containing the *anchor date*. We render
     only the days belonging to that anchor month here. */
  const visibleDays = useMemo(() => {
    if (!month) return [] as MonthResponse["days"];
    return month.days.filter((d) => {
      const im = isoMonth(d.date);
      return im.year === month.year && im.month === month.month;
    });
  }, [month]);
  const cells = useMemo(
    () => (month && visibleDays.length > 0 ? buildMonthCells(visibleDays) : []),
    [month, visibleDays]
  );

  if (!month) {
    return <EmptyState text={busy ? "Loading month…" : "Compute the month to fill the calendar."} />;
  }

  return (
    <section className="calendar" aria-busy={busy}>
      <div className="weekday-row" aria-hidden>
        {WEEKDAY_HEADERS.map((d) => (
          <span key={d}>{d}</span>
        ))}
      </div>
      <div className="month-grid">
        {cells.map((cell, idx) =>
          cell ? (
            <DayCell
              key={cell.date}
              day={cell}
              isAnchor={cell.date === anchorDate}
              isToday={cell.date === todayIso}
            />
          ) : (
            <div key={`pad-${idx}`} className="day-cell pad" aria-hidden />
          )
        )}
      </div>
    </section>
  );
}

function buildMonthCells(days: MonthResponse["days"]): Array<MonthResponse["days"][number] | null> {
  if (days.length === 0) return [];
  const leading = isoWeekday(days[0].date);
  const trailingWd = isoWeekday(days[days.length - 1].date);
  const trailing = 6 - trailingWd;
  return [
    ...Array<null>(leading).fill(null),
    ...days,
    ...Array<null>(trailing).fill(null)
  ];
}

function DayCell({
  day,
  isAnchor,
  isToday
}: {
  day: MonthResponse["days"][number];
  isAnchor: boolean;
  isToday: boolean;
}) {
  const dt = new Date(`${day.date}T00:00:00`);
  const cellNumber = dt.getDate();
  const weekday = dt.toLocaleString(undefined, { weekday: "short" });
  const isWeekend = dt.getDay() === 0 || dt.getDay() === 6;
  const events = transitionEventsForDate(day);
  const visible = events.slice(0, MAX_TX_PER_CELL);
  const overflow = events.length - visible.length;
  return (
    <article
      className={[
        "day-cell",
        isToday ? "today" : "",
        isAnchor && !isToday ? "anchor" : "",
        isWeekend ? "weekend" : ""
      ]
        .filter(Boolean)
        .join(" ")}
      aria-label={day.date}
    >
      <header className="day-head">
        <span className="day-num">{cellNumber}</span>
        <span className="day-weekday">{weekday}</span>
      </header>
      <div className="lead-stack">
        <p className="lead tithi">
          <span className="glyph" aria-hidden>{tithiGlyph(day.tithi_leader)}</span>
          <span className="name">{day.tithi_leader ?? "—"}</span>
        </p>
        <p className="lead nakshatra">
          <span className="glyph" aria-hidden>{NAKSHATRA_GLYPH}</span>
          <span className="name">{day.nakshatra_leader ?? "—"}</span>
        </p>
      </div>
      <TransitionList events={visible} compact />
      {overflow > 0 && <p className="overflow">+{overflow} more</p>}
    </article>
  );
}

function WeekStrip({
  days,
  todayIso,
  busy
}: {
  days: Array<MonthResponse["days"][number] | null>;
  todayIso: string;
  busy: boolean;
}) {
  if (days.length === 0) {
    return <EmptyState text={busy ? "Loading week…" : "Compute the current month to view the week."} />;
  }
  return (
    <section className="week-agenda" aria-busy={busy}>
      {days.map((day, idx) =>
        day ? (
          <WeekRow key={day.date} day={day} isToday={day.date === todayIso} />
        ) : (
          <article key={`pad-${idx}`} className="week-row pad" aria-hidden>
            <div className="week-row-head" />
            <div className="week-row-body" />
          </article>
        )
      )}
    </section>
  );
}

function WeekRow({ day, isToday }: { day: MonthResponse["days"][number]; isToday: boolean }) {
  const dt = new Date(`${day.date}T00:00:00`);
  const isWeekend = dt.getDay() === 0 || dt.getDay() === 6;
  const events = transitionEventsForDate(day);
  return (
    <article
      className={["week-row", isToday ? "today" : "", isWeekend ? "weekend" : ""]
        .filter(Boolean)
        .join(" ")}
      aria-label={day.date}
    >
      <header className="week-row-head">
        <span className="week-row-weekday">{dt.toLocaleString(undefined, { weekday: "short" })}</span>
        <span className="week-row-date">{dt.getDate()}</span>
        <span className="week-row-month">{dt.toLocaleString(undefined, { month: "short" })}</span>
      </header>
      <div className="week-row-body">
        <div className="week-row-leaders">
          <span className="lead tithi">
            <span className="glyph" aria-hidden>{tithiGlyph(day.tithi_leader)}</span>
            <span className="name">{day.tithi_leader ?? "—"}</span>
          </span>
          <span className="lead nakshatra">
            <span className="glyph" aria-hidden>{NAKSHATRA_GLYPH}</span>
            <span className="name">{day.nakshatra_leader ?? "—"}</span>
          </span>
        </div>
        {events.length > 0 ? (
          <ul className="week-row-events">
            {events.map((event, idx) => (
              <li
                key={`${event.kind}-${event.action}-${event.timeLocal}-${idx}`}
                className={`week-event ${event.kind} ${event.action}`}
              >
                <span className="glyph" aria-hidden>
                  {event.kind === "tithi" ? tithiGlyph(event.name) : NAKSHATRA_GLYPH}
                </span>
                <span className="name">
                  {event.name}
                  {event.pada ? <span className="pada">·{event.pada}</span> : null}
                </span>
                <span className={`time ${event.action === "starts" ? "start" : "end"}`}>
                  <span className="marker" aria-label={event.action} role="img" />
                  {timeOnly(event.timeLocal)}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="no-tx">No transitions on this day.</p>
        )}
      </div>
    </article>
  );
}

function TransitionList({ events, compact = false }: { events: TransitionEvent[]; compact?: boolean }) {
  if (events.length === 0) {
    return <p className="no-tx">No transitions on this day.</p>;
  }
  return (
    <ul className={`tx-list${compact ? " compact" : ""}`}>
      {events.map((event, index) => (
        <li
          key={`${event.kind}-${event.name}-${event.action}-${event.timeLocal}-${index}`}
          className={`tx-row ${event.kind} ${event.action}`}
        >
          <span className="glyph" aria-hidden>
            {event.kind === "tithi" ? tithiGlyph(event.name) : NAKSHATRA_GLYPH}
          </span>
          <span className="name">
            {event.name}
            {event.pada ? <span className="pada">·{event.pada}</span> : null}
          </span>
          <span className={`time ${event.action === "starts" ? "start" : "end"}`}>
            <span className="marker" aria-label={event.action} role="img" />
            {timeOnly(event.timeLocal)}
          </span>
        </li>
      ))}
    </ul>
  );
}

function transitionEventsForDate(day: MonthResponse["days"][number]): TransitionEvent[] {
  const events: TransitionEvent[] = [];
  const seen = new Set<string>();
  function pushFrom(intervals: MonthResponse["days"][number]["tithi_intervals"], kind: AngaKind) {
    for (const it of intervals) {
      if (localDate(it.start_local) === day.date) {
        const key = `${kind}|s|${it.name}|${it.start_local}|${it.pada ?? ""}`;
        if (!seen.has(key)) {
          seen.add(key);
          events.push({ name: it.name, kind, action: "starts", timeLocal: it.start_local, pada: it.pada });
        }
      }
      if (localDate(it.end_local) === day.date) {
        const key = `${kind}|e|${it.name}|${it.end_local}|${it.pada ?? ""}`;
        if (!seen.has(key)) {
          seen.add(key);
          events.push({ name: it.name, kind, action: "ends", timeLocal: it.end_local, pada: it.pada });
        }
      }
    }
  }
  pushFrom(day.tithi_intervals, "tithi");
  pushFrom(day.nakshatra_intervals, "nakshatra");
  events.sort((a, b) => timeOnly(a.timeLocal).localeCompare(timeOnly(b.timeLocal)));
  return events;
}

function localDate(value: string): string {
  return value.slice(0, 10);
}

function DailyView({
  panchangDay,
  snapshot,
  dayDetailNotice,
  dayDetailDegraded,
  date,
  time,
  timezone,
  live,
  onDateChange,
  onTimeChange,
  onSyncNow,
  onPrevious,
  onNext,
  onCompute,
  busy
}: {
  panchangDay: PanchangDayResponse | null;
  snapshot: SnapshotResponse | null;
  dayDetailNotice: DayDetailNotice | null;
  dayDetailDegraded: boolean;
  date: string;
  time: string;
  timezone: string;
  live: boolean;
  onDateChange: (date: string) => void;
  onTimeChange: (time: string) => void;
  onSyncNow: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onCompute: () => void;
  busy: boolean;
}) {
  if (!panchangDay || !snapshot) {
    return (
      <section className="daily">
        <DailyNav
          date={date}
          time={time}
          live={live}
          onDateChange={onDateChange}
          onTimeChange={onTimeChange}
          onSyncNow={onSyncNow}
          onPrevious={onPrevious}
          onNext={onNext}
          onCompute={onCompute}
          busy={busy}
        />
        <EmptyState text={busy ? "Loading day…" : "Pick a day to see full Panchang details."} />
      </section>
    );
  }
  const horaList = panchangDay.hora.length > 0 ? panchangDay.hora : snapshot.hora;
  const dayHoras = horaList.filter((h) => h.is_daytime);
  const nightHoras = horaList.filter((h) => !h.is_daytime);
  const karanaRange =
    snapshot.karana_start_local && snapshot.karana_end_local
      ? `${timeOnly(snapshot.karana_start_local)} – ${timeOnly(snapshot.karana_end_local)}`
      : null;
  const sunrise = jdToLocalTime(snapshot.sunrise_jd_ut, timezone);
  const sunset = jdToLocalTime(snapshot.sunset_jd_ut, timezone);
  const transitions = transitionEventsForDate({
    date: panchangDay.date,
    tithi_intervals: panchangDay.tithi_intervals,
    nakshatra_intervals: panchangDay.nakshatra_intervals
  });
  const tamil = panchangDay.tamil_calendar;
  return (
    <section className="daily" aria-busy={busy}>
      <DailyNav
        date={date}
        time={time}
        live={live}
        onDateChange={onDateChange}
        onTimeChange={onTimeChange}
        onSyncNow={onSyncNow}
        onPrevious={onPrevious}
        onNext={onNext}
        onCompute={onCompute}
        busy={busy}
      />
      {dayDetailNotice ? (
        <aside className="day-detail-notice" role="status">
          <strong className="day-detail-notice-title">{dayDetailNotice.title}</strong>
          <p className="day-detail-notice-body">{dayDetailNotice.body}</p>
        </aside>
      ) : null}
      <div className="daily-grid">
        <article className="daily-card panchang-card daily-card--full">
          <h2>Panchang</h2>
          <div className="anga-grid">
            <div className="anga-tile">
              <span className="anga-label">Tithi</span>
              <span className="anga-name">{snapshot.angas.tithi_name}</span>
              <span className="anga-meta">
                {snapshot.angas.paksha} · {snapshot.angas.paksha_day}
              </span>
            </div>
            <div className="anga-tile">
              <span className="anga-label">Nakshatra</span>
              <span className="anga-name">{snapshot.angas.nakshatra_name}</span>
              <span className="anga-meta">
                {snapshot.angas.nakshatra_name_tamil} · pada {snapshot.angas.nakshatra_pada}
              </span>
            </div>
            <div className="anga-tile">
              <span className="anga-label">Yoga</span>
              <span className="anga-name">{snapshot.angas.yoga_name}</span>
            </div>
            <div className="anga-tile">
              <span className="anga-label">Karana</span>
              <span className="anga-name">{snapshot.angas.karana_name}</span>
              {karanaRange ? <span className="anga-meta">{karanaRange}</span> : null}
            </div>
            <div className="anga-tile">
              <span className="anga-label">Rashi</span>
              <span className="anga-name">Sun · {snapshot.angas.sun_rashi_name}</span>
              <span className="anga-name">Moon · {snapshot.angas.moon_rashi_name}</span>
            </div>
            {(sunrise || sunset) && (
              <div className="anga-tile">
                <span className="anga-label">Sun</span>
                <span className="anga-name sun-time" title="Sunrise">
                  <span className="sun-glyph" aria-hidden>↑</span> {sunrise ?? "—"}
                </span>
                <span className="anga-name sun-time" title="Sunset">
                  <span className="sun-glyph" aria-hidden>↓</span> {sunset ?? "—"}
                </span>
              </div>
            )}
          </div>
          {snapshot.current_hora ? (
            <footer className="panchang-footer">
              <span className="anga-label">Hora now</span>
              <span className="footer-value">
                <strong>{snapshot.current_hora.ruler}</strong>
                <span className="footer-time">
                  {timeOnly(snapshot.current_hora.start_local)} – {timeOnly(snapshot.current_hora.end_local)}
                </span>
              </span>
            </footer>
          ) : null}
          <section className="panchang-transitions" aria-label="Tithi and Nakshatra transitions">
            <header className="panchang-section-head">
              <span className="anga-label">Transitions</span>
              <span className="panchang-section-sub">Tithi · Nakshatra start / end (local time)</span>
            </header>
            {transitions.length === 0 ? (
              <p className="muted">No transitions fall within this civil day.</p>
            ) : (
              <ol className="timeline">
                {transitions.map((tx, idx) => (
                  <li
                    key={`${tx.kind}-${tx.action}-${tx.name}-${tx.timeLocal}-${idx}`}
                    className={`timeline-item ${tx.kind} ${tx.action}`}
                  >
                    <span className="time">
                      <span className={`marker ${tx.action === "starts" ? "start" : "end"}`} aria-hidden />
                      {timeOnly(tx.timeLocal)}
                    </span>
                    <span className="name">
                      <span className="glyph" aria-hidden>
                        {tx.kind === "tithi" ? tithiGlyph(tx.name) : NAKSHATRA_GLYPH}
                      </span>{" "}
                      <b>{tx.name}</b>
                      <small className="action-badge">{tx.action}</small>
                      {tx.pada ? <small>pada {tx.pada}</small> : null}
                    </span>
                  </li>
                ))}
              </ol>
            )}
          </section>
        </article>
        <article className="daily-card">
          <h2>Tamil calendar</h2>
          {dayDetailDegraded ? (
            <p className="muted period-intro">Labels below fill in once the full local day run is available.</p>
          ) : null}
          <dl className="tamil-dl">
            <dt>Weekday</dt>
            <dd>{tamil.weekday_name_tamil}</dd>
            <dt>Solar month</dt>
            <dd>
              {tamil.solar_month_name} ({tamil.solar_month_name_tamil})
            </dd>
            <dt>Tamil year</dt>
            <dd>{tamil.tamil_year_name}</dd>
            <dt>Ayana · Ritu</dt>
            <dd>
              {tamil.ayana} · {tamil.ritu}
            </dd>
            <dt>Vaara</dt>
            <dd>
              {panchangDay.vaara_civil_local}
              {panchangDay.vaara_at_sunrise ? (
                <span className="muted-inline"> · at sunrise: {panchangDay.vaara_at_sunrise}</span>
              ) : null}
            </dd>
          </dl>
          {panchangDay.angas_at_sunrise ? (
            <p className="sunrise-angas muted">
              Angas at sunrise: {panchangDay.angas_at_sunrise.tithi_name}, {panchangDay.angas_at_sunrise.nakshatra_name}{" "}
              · pada {panchangDay.angas_at_sunrise.nakshatra_pada}
            </p>
          ) : null}
        </article>
        <article className="daily-card caution-card">
          <h2>Day cautions</h2>
          <p className="muted period-intro">Daytime Rahu Kalam, Yama Gandam, and Gulika Kalam (South Indian division).</p>
          <ul className="period-list">
            {panchangDay.inauspicious_periods.map((p) => (
              <li key={p.code} className={`period-row ${p.category}`}>
                <span className="period-name">{p.name}</span>
                <span className="period-range">
                  {timeOnly(p.start_local)} – {timeOnly(p.end_local)}
                </span>
              </li>
            ))}
          </ul>
          {panchangDay.inauspicious_periods.length === 0 ? (
            <p className="muted">
              {dayDetailDegraded
                ? "Included when the full local day calculation runs."
                : "None — sunrise or sunset could not be resolved for this place and date."}
            </p>
          ) : null}
        </article>
        <article className="daily-card bless-card">
          <h2>Auspicious daytime</h2>
          <ul className="period-list">
            {panchangDay.auspicious_periods.map((p) => (
              <li key={p.code} className={`period-row ${p.category}`}>
                <span className="period-name">{p.name}</span>
                <span className="period-range">
                  {timeOnly(p.start_local)} – {timeOnly(p.end_local)}
                </span>
              </li>
            ))}
          </ul>
          {panchangDay.auspicious_periods.length === 0 ? (
            <p className="muted">
              {dayDetailDegraded ? "Included when the full local day calculation runs." : "None for this date."}
            </p>
          ) : null}
        </article>
      </div>
      <div className="hora-grid">
        <HoraCard title="Day horas" subtitle="sunrise → sunset" rows={dayHoras} />
        <HoraCard title="Night horas" subtitle="sunset → next sunrise" rows={nightHoras} />
      </div>
    </section>
  );
}

function DailyNav({
  date,
  time,
  live,
  onDateChange,
  onTimeChange,
  onSyncNow,
  onPrevious,
  onNext,
  onCompute,
  busy
}: {
  date: string;
  time: string;
  live: boolean;
  onDateChange: (date: string) => void;
  onTimeChange: (time: string) => void;
  onSyncNow: () => void;
  onPrevious: () => void;
  onNext: () => void;
  onCompute: () => void;
  busy: boolean;
}) {
  return (
    <div className="daily-nav">
      <button type="button" className="nav-btn" onClick={onPrevious} title="Previous day" aria-label="Previous day">
        ‹
      </button>
      <label>
        Day
        <input type="date" value={date} onChange={(e) => onDateChange(e.target.value)} />
      </label>
      <label className="snapshot-time">
        <span className="snapshot-time-label">
          Snapshot time
          {live ? (
            <span className="live-badge" title="Auto-updating with the current time">Live</span>
          ) : (
            <button
              type="button"
              className="now-btn"
              onClick={onSyncNow}
              title="Snap to the current time and resume live updates"
            >
              Now
            </button>
          )}
        </span>
        <input type="time" step="1" value={time} onChange={(e) => onTimeChange(e.target.value)} />
      </label>
      <button type="button" className="nav-btn" onClick={onNext} title="Next day" aria-label="Next day">
        ›
      </button>
      <button type="button" className="pri" onClick={onCompute} disabled={busy}>
        {busy ? "…" : "Refresh"}
      </button>
    </div>
  );
}

function HoraCard({
  title,
  subtitle,
  rows
}: {
  title: string;
  subtitle: string;
  rows: SnapshotResponse["hora"];
}) {
  return (
    <article className="hora-card">
      <h3>
        {title}
        <span className="hora-sub">{subtitle}</span>
      </h3>
      {rows.length > 0 ? (
        <ul className="hora-list">
          {rows.map((h) => (
            <li key={`${title}-${h.index}`}>
              <span className="idx">{String(h.index + 1).padStart(2, "0")}</span>
              <span className="ruler">{h.ruler}</span>
              <span className="range">
                {timeOnly(h.start_local)} – {timeOnly(h.end_local)}
              </span>
            </li>
          ))}
        </ul>
      ) : (
        <p className="muted">Unavailable for this location/date.</p>
      )}
    </article>
  );
}

function EmptyState({ text }: { text: string }) {
  return (
    <section className="empty">
      <span className="glyph" aria-hidden>
        ✧
      </span>
      <p>{text}</p>
    </section>
  );
}
