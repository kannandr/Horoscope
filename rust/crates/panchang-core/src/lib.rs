//! Deterministic Panchang engine for API, UI, and MCP callers.

mod angas;
mod ayanamsha;
mod boundaries;
mod day_segments;
mod ephemeris;
mod hora;
mod meeus_tables;
mod muhurta;
mod names;
mod rise_set;
mod time;
mod types;

use chrono::Datelike;

pub use crate::day_segments::{civil_day, month, panchang_day};
pub use crate::muhurta::search_muhurta;
pub use crate::time::{datetime_utc_from_jd, julian_day_ut, local_to_utc};
pub use crate::types::*;

pub fn snapshot(req: SnapshotRequest) -> Result<SnapshotResponse, PanchangError> {
    types::validate_observer(req.latitude, req.longitude)?;
    let tz = time::parse_timezone(&req.timezone)?;
    let local = time::parse_local_datetime(&req.when_local)?;
    let utc = time::local_to_utc(local, tz);
    let jd = time::julian_day_ut(utc);
    let engine = req.engine.unwrap_or_default();
    let ayanamsha = req.ayanamsha.unwrap_or_default();
    let trop = ephemeris::apparent_tropical_longitudes(jd, engine);
    let ay_deg = ayanamsha::delta_deg(jd, ayanamsha);
    let sun_sidereal_deg = ephemeris::reduce_deg(trop.sun_deg - ay_deg);
    let moon_sidereal_deg = ephemeris::reduce_deg(trop.moon_lon_deg - ay_deg);
    let angas = angas::compute_angas(sun_sidereal_deg, moon_sidereal_deg);
    let sunrise_jd_ut = rise_set::find_prev_sunrise_jd(jd, req.latitude, req.longitude);
    let sunset_jd_ut = sunrise_jd_ut
        .and_then(|sr| rise_set::find_next_sunset_jd(sr + 1e-6, req.latitude, req.longitude));
    let next_sunrise_jd_ut = sunset_jd_ut
        .and_then(|ss| rise_set::find_next_sunrise_jd(ss + 1e-6, req.latitude, req.longitude));
    let hora = match (sunrise_jd_ut, sunset_jd_ut, next_sunrise_jd_ut) {
        (Some(sr), Some(ss), Some(ns)) => hora::build_hora_table(sr, ss, ns, tz),
        _ => Vec::new(),
    };
    let current_hora = hora
        .iter()
        .find(|h| h.jd_start <= jd && jd < h.jd_end)
        .cloned();
    let karana_start_jd_ut = boundaries::prev_karana_start_jd(jd, ayanamsha, engine);
    let karana_end_jd_ut = boundaries::next_karana_end_jd(jd, ayanamsha, engine);

    Ok(SnapshotResponse {
        jd_ut: jd,
        engine,
        ayanamsha,
        ayanamsha_deg: ay_deg,
        sun_tropical_deg: trop.sun_deg,
        moon_tropical_deg: trop.moon_lon_deg,
        moon_tropical_lat_deg: trop.moon_lat_deg,
        sun_sidereal_deg,
        moon_sidereal_deg,
        angas,
        vaara_civil_local: names::weekday_name(
            utc.with_timezone(&tz).weekday().num_days_from_monday() as usize,
        )
        .to_string(),
        sunrise_jd_ut,
        sunset_jd_ut,
        next_sunrise_jd_ut,
        tithi_start_jd_ut: boundaries::prev_tithi_start_jd(jd, ayanamsha, engine),
        next_tithi_end_jd: boundaries::next_tithi_end_jd(jd, ayanamsha, engine),
        next_nakshatra_end_jd: boundaries::next_nakshatra_end_jd(jd, ayanamsha, engine),
        next_yoga_end_jd: boundaries::next_yoga_end_jd(jd, ayanamsha, engine),
        karana_start_jd_ut,
        karana_end_jd_ut,
        karana_start_local: karana_start_jd_ut.map(|x| time::local_iso_from_jd(x, tz)),
        karana_end_local: karana_end_jd_ut.map(|x| time::local_iso_from_jd(x, tz)),
        hora,
        current_hora,
    })
}
