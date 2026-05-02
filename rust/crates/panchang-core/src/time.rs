use chrono::{DateTime, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

use crate::types::PanchangError;

pub fn parse_timezone(name: &str) -> Result<Tz, PanchangError> {
    name.parse::<Tz>()
        .map_err(|_| PanchangError::InvalidTimezone(name.to_string()))
}

pub fn parse_local_datetime(value: &str) -> Result<NaiveDateTime, PanchangError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.naive_local());
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S"))
        .map_err(|_| PanchangError::InvalidDateTime(value.to_string()))
}

pub fn parse_date(value: &str) -> Result<NaiveDate, PanchangError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| PanchangError::InvalidDate(value.to_string()))
}

pub fn local_to_utc(local: NaiveDateTime, zone: Tz) -> DateTime<Utc> {
    match zone.from_local_datetime(&local) {
        LocalResult::Single(dt) => dt.with_timezone(&Utc),
        LocalResult::Ambiguous(a, _) => a.with_timezone(&Utc),
        LocalResult::None => zone.from_utc_datetime(&local).with_timezone(&Utc),
    }
}

pub fn julian_day_ut(utc: DateTime<Utc>) -> f64 {
    let utc = utc.with_timezone(&Utc);
    let mut y = utc.year();
    let mut m = utc.month() as i32;
    let d = utc.day() as i32;
    let day_fraction = (utc.hour() as f64
        + utc.minute() as f64 / 60.0
        + utc.second() as f64 / 3600.0
        + utc.timestamp_subsec_micros() as f64 / 3_600_000_000.0)
        / 24.0;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let a = y.div_euclid(100);
    let b = 2 - a + a.div_euclid(4);
    let jd0 = (365.25 * (y + 4716) as f64).floor()
        + (30.6001 * (m + 1) as f64).floor()
        + d as f64
        + b as f64
        - 1524.5;
    jd0 + day_fraction
}

pub fn datetime_utc_from_jd(jd: f64) -> DateTime<Utc> {
    let unix = ((jd - 2440587.5) * 86400.0).round() as i64;
    Utc.timestamp_opt(unix, 0)
        .single()
        .expect("Julian day converted outside chrono timestamp range")
}

pub fn local_iso_from_jd(jd: f64, zone: Tz) -> String {
    datetime_utc_from_jd(jd)
        .with_timezone(&zone)
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
}

pub fn jd_from_local_midnight(date: NaiveDate, zone: Tz) -> f64 {
    let midnight = date
        .and_hms_opt(0, 0, 0)
        .expect("valid midnight for NaiveDate");
    julian_day_ut(local_to_utc(midnight, zone))
}

use chrono::{Datelike, Timelike};
