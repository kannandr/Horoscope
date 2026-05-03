use chrono::{Duration, NaiveDate};
use panchang_core::{
    julian_day_ut, local_iso_from_jd, local_to_utc, parse_date, parse_timezone, validate_observer,
    DayPeriod, PanchangDayMode, PanchangDayRequest, SnapshotRequest,
};

use crate::client::PanchangClient;
use crate::types::{
    EngineError, MuhurtaSearchRequest, MuhurtaSearchResponse, MuhurtaWindow,
};

const FAVORABLE_NAKSHATRAS: &[&str] = &[
    "Rohini",
    "Mrigashira",
    "Punarvasu",
    "Pushya",
    "Hasta",
    "Chitra",
    "Anuradha",
    "Shravana",
    "Revati",
];
const AVOID_TITHIS: &[&str] = &["Chaturthi", "Ashtami", "Navami", "Chaturdashi", "Amavasya"];

fn date_range(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut d = start;
    let mut out = Vec::new();
    while d <= end && out.len() < 62 {
        out.push(d);
        d += Duration::days(1);
    }
    out
}

fn overlaps(start_jd: f64, end_jd: f64, period: &DayPeriod) -> bool {
    start_jd < period.jd_end && end_jd > period.jd_start
}

/// Search auspicious time windows by asking a [`PanchangClient`] for the
/// underlying Panchang data and applying the rule set.
///
/// In production `client` is an `McpPanchangClient`, which means every
/// scoring iteration round-trips the network. That is intentional: it is
/// the same surface a learned model would consume.
pub async fn search_muhurta<C: PanchangClient + ?Sized>(
    client: &C,
    req: MuhurtaSearchRequest,
) -> Result<MuhurtaSearchResponse, EngineError> {
    validate_observer(req.latitude, req.longitude)?;
    let tz = parse_timezone(&req.timezone)?;
    let start = parse_date(&req.date_start)?;
    let end = parse_date(&req.date_end)?;
    let min_minutes = req.min_duration_minutes.unwrap_or(45).max(15);
    let preset = req
        .purpose_preset
        .clone()
        .unwrap_or_else(|| "south_indian_tamil_general".to_string());
    let mut windows = Vec::new();

    for date in date_range(start, end) {
        let day = client
            .panchang_day(PanchangDayRequest {
                date: date.to_string(),
                timezone: req.timezone.clone(),
                latitude: req.latitude,
                longitude: req.longitude,
                day_mode: Some(PanchangDayMode::CivilMidnight),
                ayanamsha: req.ayanamsha,
                engine: req.engine,
            })
            .await?;

        for hour in 6..19 {
            let local = date
                .and_hms_opt(hour, 0, 0)
                .expect("valid whole-hour local time");
            let snap = client
                .snapshot(SnapshotRequest {
                    when_local: local.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    timezone: req.timezone.clone(),
                    latitude: req.latitude,
                    longitude: req.longitude,
                    ayanamsha: req.ayanamsha,
                    engine: req.engine,
                })
                .await?;
            let mut score = 50;
            let mut reasons = Vec::new();
            let mut exclusions = Vec::new();
            if FAVORABLE_NAKSHATRAS.contains(&snap.angas.nakshatra_name.as_str()) {
                score += 20;
                reasons.push(format!(
                    "Favorable nakshatra: {}",
                    snap.angas.nakshatra_name
                ));
            }
            if snap.angas.paksha == "shukla" {
                score += 10;
                reasons.push("Shukla paksha is preferred by this preset".to_string());
            }
            if let Some(h) = &snap.current_hora {
                if ["Jupiter", "Venus", "Mercury", "Moon"].contains(&h.ruler.as_str()) {
                    score += 10;
                    reasons.push(format!("Supportive hora: {}", h.ruler));
                } else {
                    exclusions.push(format!("Less preferred hora: {}", h.ruler));
                }
            }
            if AVOID_TITHIS.contains(&snap.angas.tithi_name.as_str()) {
                score -= 35;
                exclusions.push(format!(
                    "Avoided tithi in Tamil preset: {}",
                    snap.angas.tithi_name
                ));
            }
            let start_jd = julian_day_ut(local_to_utc(local, tz));
            let end_jd = start_jd + min_minutes as f64 / 1440.0;
            let bad_periods = day
                .inauspicious_periods
                .iter()
                .filter(|period| overlaps(start_jd, end_jd, period))
                .map(|period| period.name.as_str())
                .collect::<Vec<_>>();
            if !bad_periods.is_empty() {
                score -= 45;
                exclusions.push(format!("Overlaps {}", bad_periods.join(", ")));
            }
            if day
                .auspicious_periods
                .iter()
                .any(|period| overlaps(start_jd, end_jd, period))
            {
                score += 15;
                reasons.push("Overlaps Abhijit Muhurta".to_string());
            }
            if let (Some(sr), Some(ss)) = (snap.sunrise_jd_ut, snap.sunset_jd_ut) {
                if start_jd < sr || end_jd > ss {
                    score -= 50;
                    exclusions.push("Outside sunrise-to-sunset bounds".to_string());
                }
            }
            if score >= 55 {
                windows.push(MuhurtaWindow {
                    start_local: local_iso_from_jd(start_jd, tz),
                    end_local: local_iso_from_jd(end_jd, tz),
                    duration_minutes: min_minutes,
                    score,
                    label: if score >= 80 {
                        "Strong candidate"
                    } else {
                        "Usable candidate"
                    }
                    .to_string(),
                    reasons,
                    exclusions,
                });
            }
        }
    }
    windows.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.start_local.cmp(&b.start_local))
    });
    windows.truncate(24);
    Ok(MuhurtaSearchResponse {
        preset,
        timezone: req.timezone,
        windows,
    })
}
