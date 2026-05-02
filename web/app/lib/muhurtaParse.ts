/** Lightweight NL → structured search for Tamil/South Indian auspicious windows.
 * Maps informal phrases to API fields; the Rust engine still runs one preset
 * (`south_indian_tamil_general`) — we document that in `interpretation`. */

function addDaysIso(iso: string, days: number): string {
  const [y, m, d] = iso.split("-").map(Number);
  const dt = new Date(Date.UTC(y, m - 1, d));
  dt.setUTCDate(dt.getUTCDate() + days);
  const yy = dt.getUTCFullYear();
  const mm = String(dt.getUTCMonth() + 1).padStart(2, "0");
  const dd = String(dt.getUTCDate()).padStart(2, "0");
  return `${yy}-${mm}-${dd}`;
}

/** Common nakshatra spellings for detecting user mentions (substring match). */
const NAKSHATRA_HINTS = [
  "Ashwini",
  "Bharani",
  "Krittika",
  "Rohini",
  "Mrigashira",
  "Mrigasira",
  "Ardra",
  "Punarvasu",
  "Pushya",
  "Ashlesha",
  "Magha",
  "Purva Phalguni",
  "Uttara Phalguni",
  "Hasta",
  "Chitra",
  "Swati",
  "Vishakha",
  "Anuradha",
  "Jyeshtha",
  "Mula",
  "Purva Ashadha",
  "Uttara Ashadha",
  "Shravana",
  "Dhanishta",
  "Shatabhisha",
  "Purva Bhadrapada",
  "Uttara Bhadrapada",
  "Revati"
];

export type ParsedMuhurtaIntent = {
  date_start: string;
  date_end: string;
  min_duration_minutes: number;
  purpose_preset: string;
  interpretation: string;
};

/** Pull explicit YYYY-MM-DD ranges from free text. */
function extractIsoDates(text: string): string[] {
  const re = /\b(\d{4}-\d{2}-\d{2})\b/g;
  const out: string[] = [];
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    out.push(m[1]);
  }
  return out;
}

export function parseMuhurtaQuery(text: string, anchorIso: string): ParsedMuhurtaIntent {
  const trimmed = text.trim();
  const lower = trimmed.toLowerCase();

  const isoDates = extractIsoDates(trimmed);
  let date_start = anchorIso;
  let date_end = addDaysIso(anchorIso, 6);

  if (isoDates.length >= 2) {
    const [a, b] = isoDates.sort();
    date_start = a;
    date_end = b >= a ? b : a;
  } else if (isoDates.length === 1) {
    date_start = isoDates[0];
    date_end = addDaysIso(isoDates[0], 6);
  }

  let spanDays =
    Math.round(
      (new Date(date_end + "T12:00:00Z").getTime() - new Date(date_start + "T12:00:00Z").getTime()) /
        86_400_000
    ) + 1;
  spanDays = Math.min(62, Math.max(1, spanDays));

  const nextDaysMatch = lower.match(/next\s+(\d+)\s+days?/);
  const thisDaysMatch = lower.match(/(?:this|these)\s+(\d+)\s+days?/);
  if (nextDaysMatch && !isoDates.length) {
    const n = Math.min(62, Math.max(1, parseInt(nextDaysMatch[1], 10)));
    date_start = anchorIso;
    date_end = addDaysIso(anchorIso, n - 1);
    spanDays = n;
  } else if (thisDaysMatch && !isoDates.length) {
    const n = Math.min(62, Math.max(1, parseInt(thisDaysMatch[1], 10)));
    date_end = addDaysIso(date_start, n - 1);
    spanDays = n;
  } else if (!isoDates.length) {
    if (/\b(two weeks|2\s*weeks|14\s*days?)\b/i.test(trimmed)) {
      date_end = addDaysIso(date_start, 13);
      spanDays = 14;
    } else if (/\b(three days|3\s*days?)\b/i.test(trimmed)) {
      date_end = addDaysIso(date_start, 2);
      spanDays = 3;
    } else if (/\b(week|7\s*days?)\b/i.test(trimmed)) {
      date_end = addDaysIso(date_start, 6);
      spanDays = 7;
    }
  }

  let min_duration_minutes = 30;
  const hourMatch = lower.match(/(\d+)\s*(?:hours?|hrs?)\b/);
  if (hourMatch) {
    min_duration_minutes = Math.max(15, parseInt(hourMatch[1], 10) * 60);
  }
  const minMatch = lower.match(/(\d+)\s*mins?\b/);
  if (minMatch && !hourMatch) {
    min_duration_minutes = Math.max(15, parseInt(minMatch[1], 10));
  }

  const purpose_preset = "south_indian_tamil_general";

  const bits: string[] = [
    `Searching ${date_start} → ${date_end} (${spanDays} civil days), minimum block ${min_duration_minutes} minutes.`,
    "Engine uses the South Indian / Tamil general preset (favorable nakshatras, shukla bias, hora & Rahu-kalam avoidance are scored in the backend)."
  ];

  if (/wedding|marriage|vivah|kalyan|engagement/i.test(trimmed)) {
    bits.push("You mentioned marriage / wedding — same mathematical preset; treat results as general guidance.");
  }
  if (/travel|journey|trip|flight/i.test(trimmed)) {
    bits.push("You mentioned travel — results rank daytime windows inside sunrise–sunset.");
  }
  if (/griha|house\s*warming|new\s*home/i.test(trimmed)) {
    bits.push("You mentioned house-related activity — same preset applies.");
  }

  const mentioned = NAKSHATRA_HINTS.filter((n) => lower.includes(n.toLowerCase()));
  if (mentioned.length) {
    bits.push(
      `You named: ${mentioned.join(", ")}. Scoring already favors common auspicious nakshatras; watch the “reasons” on each window for matches.`
    );
  }

  return {
    date_start,
    date_end,
    min_duration_minutes,
    purpose_preset,
    interpretation: bits.join(" ")
  };
}
