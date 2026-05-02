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
};

export type MonthResponse = {
  year: number;
  month: number;
  timezone: string;
  days: Array<{
    date: string;
    tithi_leader?: string | null;
    nakshatra_leader?: string | null;
    tithi_intervals: Segment[];
    nakshatra_intervals: Segment[];
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

/** Time portion (HH:MM:SS) extracted from an ISO local-datetime string. */
export function timeOnly(value: string): string {
  const match = value.match(/T(\d\d:\d\d:\d\d)/);
  return match?.[1] ?? value;
}
