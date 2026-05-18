export type EngineId = "meeus" | "surya_mean";
export type AyanamshaId = "lahiri" | "lahiri_alt_stub" | "raman";

export type Observer = {
  timezone: string;
  latitude: number;
  longitude: number;
  ayanamsha?: AyanamshaId;
  engine?: EngineId;
};

export type Segment = {
  name: string;
  start_jd_ut: number;
  end_jd_ut: number;
  start_local: string;
  end_local: string;
  clipped_start_jd_ut?: number;
  clipped_end_jd_ut?: number;
  clipped_start_local?: string;
  clipped_end_local?: string;
  starts_before_window?: boolean;
  ends_after_window?: boolean;
  pada?: number | null;
};

export type SnapshotResponse = {
  jd_ut: number;
  angas: {
    tithi_name: string;
    paksha: string;
    paksha_day: number;
    nakshatra_name: string;
    nakshatra_name_tamil: string;
    nakshatra_pada: number;
    yoga_name: string;
    karana_name: string;
    sun_rashi_name: string;
    moon_rashi_name: string;
  };
  vaara_civil_local: string;
  sunrise_jd_ut?: number | null;
  sunset_jd_ut?: number | null;
  next_tithi_end_jd?: number | null;
  next_nakshatra_end_jd?: number | null;
  karana_start_jd_ut?: number | null;
  karana_end_jd_ut?: number | null;
  karana_start_local?: string | null;
  karana_end_local?: string | null;
  current_hora?: { ruler: string; start_local: string; end_local: string } | null;
  hora: Array<{ index: number; ruler: string; is_daytime: boolean; start_local: string; end_local: string }>;
};

export type CivilDayResponse = {
  date: string;
  timezone: string;
  tithi_intervals: Segment[];
  nakshatra_intervals: Segment[];
  yoga_intervals?: Segment[];
  karana_intervals?: Segment[];
};

export type DayPeriod = {
  code: string;
  name: string;
  category: string;
  jd_start: number;
  jd_end: number;
  start_local: string;
  end_local: string;
  source: string;
};

export type PanchangDayResponse = {
  date: string;
  timezone: string;
  day_mode: "civil_midnight" | "sunrise_day";
  day_start_jd_ut: number;
  day_end_jd_ut: number;
  day_start_local: string;
  day_end_local: string;
  sunrise_local?: string | null;
  sunset_local?: string | null;
  next_sunrise_local?: string | null;
  vaara_civil_local: string;
  vaara_at_sunrise?: string | null;
  angas_at_sunrise?: SnapshotResponse["angas"] | null;
  tamil_calendar: {
    solar_month_index: number;
    solar_month_name: string;
    solar_month_name_tamil: string;
    tamil_year_index: number;
    tamil_year_name: string;
    ayana: string;
    ritu: string;
    weekday_name_tamil: string;
  };
  tithi_intervals: Segment[];
  nakshatra_intervals: Segment[];
  yoga_intervals: Segment[];
  karana_intervals: Segment[];
  hora: SnapshotResponse["hora"];
  inauspicious_periods: DayPeriod[];
  auspicious_periods: DayPeriod[];
};

export type MonthResponse = {
  year: number;
  month: number;
  timezone: string;
  days: Array<{
    date: string;
    tithi_leader?: string | null;
    nakshatra_leader?: string | null;
    yoga_leader?: string | null;
    karana_leader?: string | null;
    tithi_intervals: Segment[];
    nakshatra_intervals: Segment[];
    yoga_intervals?: Segment[];
    karana_intervals?: Segment[];
  }>;
};

export type MuhurtaResponse = {
  preset: string;
  timezone: string;
  windows: Array<{
    start_local: string;
    end_local: string;
    duration_minutes: number;
    score: number;
    label: string;
    reasons: string[];
    exclusions: string[];
  }>;
};

export type HoroscopeBody = {
  sidereal_longitude_deg: number;
  rashi_index: number;
  rashi_name: string;
  nakshatra_index: number;
  nakshatra_name: string;
  nakshatra_pada: number;
  retrograde?: boolean;
  rashi_name_tamil?: string;
  nakshatra_name_tamil?: string;
};

export type HoroscopeResponse = {
  schema_version: string;
  kind: "south_indian_natal_chart";
  birth: {
    birth_local: string;
    timezone: string;
    latitude: number;
    longitude: number;
    utc_iso: string;
    jd_ut: number;
  };
  frame: {
    ayanamsha: AyanamshaId;
    ayanamsha_deg: number;
    engine: EngineId;
    sidereal_zodiac: string;
    lunar_node_policy?: string;
    slow_planet_ephemeris?: string;
  };
  lagna: HoroscopeBody;
  grahas: Record<
    "sun" | "moon" | "mars" | "mercury" | "jupiter" | "venus" | "saturn" | "rahu" | "ketu",
    HoroscopeBody
  >;
  panchang_at_birth: {
    vaara: string;
    tithi_name: string;
    yoga_name: string;
    karana_name: string;
    paksha: string;
    sunrise_local?: string | null;
    sunset_local?: string | null;
  };
  tamil_calendar_hint: {
    solar_month_name: string;
    solar_month_name_tamil: string;
    tamil_year_name: string;
    weekday_name_tamil: string;
  };
  dasha_bhukti: {
    system: string;
    moon_at_birth: {
      nakshatra_index: number;
      nakshatra_name: string;
      starting_mahadasha_lord: string;
      balance_of_starting_mahadasha_at_birth_days: number;
    };
    window: {
      as_of_local: string;
      horizon_end_local: string;
      horizon_years_after_as_of: number;
      timezone: string;
    };
    mahadashas: Array<{
      lord: string;
      lord_display_en: string;
      lord_display_ta: string;
      start_local: string;
      end_local: string;
      antardashas: Array<{
        lord: string;
        lord_display_en: string;
        lord_display_ta: string;
        start_local: string;
        end_local: string;
      }>;
    }>;
  };
};

/** Time portion (HH:MM:SS) extracted from an ISO local-datetime string. */
export function timeOnly(value: string): string {
  const match = value.match(/T(\d\d:\d\d:\d\d)/);
  return match?.[1] ?? value;
}

/** Date portion (YYYY-MM-DD) extracted from an ISO local-datetime string. */
export function dateOnly(value: string): string {
  return value.slice(0, 10);
}

/** Format a YYYY-MM-DD as a long, locale-friendly label like "Sat, May 2 2026". */
export function formatDateLong(iso: string): string {
  const m = iso.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (!m) return iso;
  const dt = new Date(Date.UTC(Number(m[1]), Number(m[2]) - 1, Number(m[3]), 12, 0, 0));
  return dt.toLocaleDateString(undefined, {
    timeZone: "UTC",
    weekday: "short",
    day: "numeric",
    month: "short",
    year: "numeric"
  });
}
