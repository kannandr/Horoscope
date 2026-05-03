"use client";

import { useState } from "react";
import type { MuhurtaResponse, Observer } from "@/app/lib/api";
import { dateOnly, formatDateLong, timeOnly } from "@/app/lib/api";
import { parseMuhurtaQuery } from "@/app/lib/muhurtaParse";

export type AuspiciousNotice = {
  title: string;
  body: string;
};

export function AuspiciousView({
  anchorDate,
  observer,
  result,
  busy,
  notice,
  onSearch,
  onAnchorDateChange
}: {
  anchorDate: string;
  observer: Observer;
  result: MuhurtaResponse | null;
  busy: boolean;
  notice: AuspiciousNotice | null;
  onSearch: (query: string) => void;
  onAnchorDateChange: (iso: string) => void;
}) {
  const [query, setQuery] = useState(
    "Find auspicious daytime windows this week for an important event. Prefer at least 45 minutes."
  );
  const [lastInterpretation, setLastInterpretation] = useState<string | null>(null);

  function submit(e: React.FormEvent) {
    e.preventDefault();
    const parsed = parseMuhurtaQuery(query, anchorDate);
    setLastInterpretation(parsed.interpretation);
    onSearch(query);
  }

  return (
    <section className="auspicious-view" aria-busy={busy}>
      <header className="auspicious-header">
        <h2 className="auspicious-title">Auspicious times</h2>
        <p className="auspicious-lede muted">
          Describe your event in plain English (nakshatra names, wedding, travel, how many days to search).
          Scoring runs in the Muhurta service; each candidate asks the Panchang calculation core for times and
          angas — in production that core is reached via the same MCP server agents use, not a third-party API.
        </p>
      </header>

      <form className="auspicious-form" onSubmit={submit}>
        <label className="auspicious-query-label">
          <span className="anga-label">Ask</span>
          <textarea
            className="auspicious-query"
            rows={4}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder='Example: "Wedding in Rohini or Mrigashira — search next 10 days, need at least 1 hour clear"'
            spellCheck={true}
          />
        </label>
        <div className="auspicious-actions">
          <label className="auspicious-start-date">
            <span className="anga-label">Start date</span>
            <input
              type="date"
              value={anchorDate}
              onChange={(e) => onAnchorDateChange(e.target.value)}
            />
          </label>
          <button type="submit" className="pri auspicious-submit" disabled={busy}>
            {busy ? "Searching…" : "Find windows"}
          </button>
        </div>
      </form>

      {lastInterpretation ? (
        <aside className="auspicious-interpretation" role="note">
          <span className="anga-label">How we read that</span>
          <p>{lastInterpretation}</p>
        </aside>
      ) : null}

      {notice ? (
        <aside className="day-detail-notice" role="alert" aria-live="polite">
          <span className="day-detail-notice-title">{notice.title}</span>
          <span className="day-detail-notice-body">{notice.body}</span>
        </aside>
      ) : null}

      <div className="auspicious-meta muted">
        <span>
          Location: {observer.latitude.toFixed(4)}°, {observer.longitude.toFixed(4)}° · {observer.timezone}
        </span>
      </div>

      {result && result.windows.length === 0 && !notice ? (
        <p className="auspicious-empty muted">No scored windows in this range — try widening dates or lowering the minimum duration.</p>
      ) : null}

      {result && result.windows.length > 0 ? (
        <ol className="auspicious-window-groups">
          {groupWindowsByDate(result.windows).map((group) => (
            <li key={group.date} className="auspicious-window-group">
              <h3 className="auspicious-window-group-head">
                <span className="auspicious-window-date">{formatDateLong(group.date)}</span>
                <span className="auspicious-window-count">
                  {group.windows.length} window{group.windows.length === 1 ? "" : "s"}
                </span>
              </h3>
              <ul className="auspicious-windows">
                {group.windows.map((w, i) => (
                  <li
                    key={`${w.start_local}-${w.score}-${i}`}
                    className="auspicious-window-card"
                  >
                    <div className="auspicious-window-head">
                      <span className="auspicious-window-label">{w.label}</span>
                      <span className="auspicious-window-score">score {w.score}</span>
                    </div>
                    <div className="auspicious-window-time">
                      <span className="auspicious-window-date-inline">{formatDateLong(dateOnly(w.start_local))}</span>
                      <span className="auspicious-window-time-range">
                        {timeOnly(w.start_local)} – {timeOnly(w.end_local)}
                      </span>
                      <span className="auspicious-window-duration">· {w.duration_minutes} min</span>
                    </div>
                    {w.reasons.length > 0 ? (
                      <ul className="auspicious-reasons">
                        {w.reasons.map((r) => (
                          <li key={r}>{r}</li>
                        ))}
                      </ul>
                    ) : null}
                    {w.exclusions.length > 0 ? (
                      <ul className="auspicious-exclusions">
                        {w.exclusions.map((x) => (
                          <li key={x}>{x}</li>
                        ))}
                      </ul>
                    ) : null}
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ol>
      ) : null}

      {!result && !busy ? (
        <p className="muted auspicious-hint">Submit the form to search from your start date.</p>
      ) : null}
    </section>
  );
}

/** Group muhurta windows by their local civil date so users can scan day-by-day. */
function groupWindowsByDate(windows: MuhurtaResponse["windows"]): Array<{
  date: string;
  windows: MuhurtaResponse["windows"];
}> {
  const buckets = new Map<string, MuhurtaResponse["windows"]>();
  for (const w of windows) {
    const key = dateOnly(w.start_local);
    const list = buckets.get(key) ?? [];
    list.push(w);
    buckets.set(key, list);
  }
  return Array.from(buckets.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([date, ws]) => ({ date, windows: ws }));
}
