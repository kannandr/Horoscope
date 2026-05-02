import type { CivilDayResponse, PanchangDayResponse, SnapshotResponse } from "@/app/lib/api";

/** When `POST /v1/panchang/day` is unavailable (older API), build a day-shaped
 * payload from `civil-day` + snapshot so the daily UI can still render. */
export function panchangDayFromCivilAndSnapshot(
  civil: CivilDayResponse,
  snap: SnapshotResponse
): PanchangDayResponse {
  return {
    date: civil.date,
    timezone: civil.timezone,
    day_mode: "civil_midnight",
    day_start_jd_ut: 0,
    day_end_jd_ut: 0,
    day_start_local: `${civil.date}T00:00:00`,
    day_end_local: `${civil.date}T23:59:59`,
    sunrise_local: null,
    sunset_local: null,
    next_sunrise_local: null,
    vaara_civil_local: snap.vaara_civil_local,
    vaara_at_sunrise: null,
    angas_at_sunrise: null,
    tamil_calendar: {
      solar_month_index: 0,
      solar_month_name: "—",
      solar_month_name_tamil: "—",
      tamil_year_index: 0,
      tamil_year_name: "—",
      ayana: "—",
      ritu: "—",
      weekday_name_tamil: "—"
    },
    tithi_intervals: civil.tithi_intervals,
    nakshatra_intervals: civil.nakshatra_intervals,
    yoga_intervals: civil.yoga_intervals ?? [],
    karana_intervals: civil.karana_intervals ?? [],
    hora: [],
    inauspicious_periods: [],
    auspicious_periods: []
  };
}

/** Last resort: day + civil both failed but snapshot succeeded — show Panchang/hora from snapshot only. */
export function panchangDayFromSnapshotOnly(
  date: string,
  timezone: string,
  snap: SnapshotResponse
): PanchangDayResponse {
  return {
    date,
    timezone,
    day_mode: "civil_midnight",
    day_start_jd_ut: 0,
    day_end_jd_ut: 0,
    day_start_local: `${date}T00:00:00`,
    day_end_local: `${date}T23:59:59`,
    sunrise_local: null,
    sunset_local: null,
    next_sunrise_local: null,
    vaara_civil_local: snap.vaara_civil_local,
    vaara_at_sunrise: null,
    angas_at_sunrise: null,
    tamil_calendar: {
      solar_month_index: 0,
      solar_month_name: "—",
      solar_month_name_tamil: "—",
      tamil_year_index: 0,
      tamil_year_name: "—",
      ayana: "—",
      ritu: "—",
      weekday_name_tamil: "—"
    },
    tithi_intervals: [],
    nakshatra_intervals: [],
    yoga_intervals: [],
    karana_intervals: [],
    hora: [],
    inauspicious_periods: [],
    auspicious_periods: []
  };
}
