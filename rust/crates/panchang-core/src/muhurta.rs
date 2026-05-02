use chrono::{Duration, NaiveDate};

use crate::time;
use crate::types::{
    validate_observer, MuhurtaSearchRequest, MuhurtaSearchResponse, MuhurtaWindow, PanchangError,
    SnapshotRequest,
};

const FAVORABLE_NAKSHATRAS: &[&str] = &[
    "Rohini", "Mrigashira", "Punarvasu", "Pushya", "Hasta", "Chitra", "Anuradha", "Shravana",
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

pub fn search_muhurta(req: MuhurtaSearchRequest) -> Result<MuhurtaSearchResponse, PanchangError> {
    validate_observer(req.latitude, req.longitude)?;
    let tz = time::parse_timezone(&req.timezone)?;
    let start = time::parse_date(&req.date_start)?;
    let end = time::parse_date(&req.date_end)?;
    let min_minutes = req.min_duration_minutes.unwrap_or(45).max(15);
    let preset = req.purpose_preset.clone().unwrap_or_else(|| "south_indian_tamil_general".to_string());
    let mut windows = Vec::new();

    for date in date_range(start, end) {
        for hour in 6..19 {
            let local = date.and_hms_opt(hour, 0, 0).expect("valid whole-hour local time");
            let snap = crate::snapshot(SnapshotRequest {
                when_local: local.format("%Y-%m-%dT%H:%M:%S").to_string(),
                timezone: req.timezone.clone(),
                latitude: req.latitude,
                longitude: req.longitude,
                ayanamsha: req.ayanamsha,
                engine: req.engine,
            })?;
            let mut score = 50;
            let mut reasons = Vec::new();
            let mut exclusions = Vec::new();
            if FAVORABLE_NAKSHATRAS.contains(&snap.angas.nakshatra_name.as_str()) {
                score += 20;
                reasons.push(format!("Favorable nakshatra: {}", snap.angas.nakshatra_name));
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
                exclusions.push(format!("Avoided tithi in Tamil preset: {}", snap.angas.tithi_name));
            }
            let start_jd = time::julian_day_ut(time::local_to_utc(local, tz));
            let end_jd = start_jd + min_minutes as f64 / 1440.0;
            if let (Some(sr), Some(ss)) = (snap.sunrise_jd_ut, snap.sunset_jd_ut) {
                if start_jd < sr || end_jd > ss {
                    score -= 50;
                    exclusions.push("Outside sunrise-to-sunset bounds".to_string());
                }
            }
            if score >= 55 {
                windows.push(MuhurtaWindow {
                    start_local: time::local_iso_from_jd(start_jd, tz),
                    end_local: time::local_iso_from_jd(end_jd, tz),
                    duration_minutes: min_minutes,
                    score,
                    label: if score >= 80 { "Strong candidate" } else { "Usable candidate" }.to_string(),
                    reasons,
                    exclusions,
                });
            }
        }
    }
    windows.sort_by(|a, b| b.score.cmp(&a.score).then(a.start_local.cmp(&b.start_local)));
    windows.truncate(24);
    Ok(MuhurtaSearchResponse { preset, timezone: req.timezone, windows })
}
