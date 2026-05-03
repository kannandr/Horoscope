//! South Indian natal chart assembly on top of [`panchang_core`].

mod output;
mod vimshottari;

use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use chrono_tz::Tz;
use serde::Deserialize;
use thiserror::Error;

pub use output::{
    BirthOut, BodyLongitudeOut, DashaBhuktiOut, ExtensionsOut, FrameOut, GrahaFullOut,
    GrahasOut, PanchangAtBirthOut, SouthIndianNatalChart, TamilCalendarHintOut, SCHEMA_VERSION,
};

use panchang_core::{
    ascendant_tropical_deg, julian_day_ut, local_iso_from_jd, local_to_utc, mean_north_node_apparent_longitude_deg,
    names, planet_geocentric_apparent_longitude_deg, planet_geocentric_longitude_retrograde,
    panchang_day, parse_local_datetime, parse_timezone, reduce_deg, snapshot, validate_observer,
    AyanamshaId, EngineId, KeplerPlanet, PanchangDayMode, PanchangDayRequest, SnapshotRequest,
};
use vimshottari::build_dasha_bhukti;
const RASHI_NAMES_TAMIL: [&str; 12] = [
    "Mesham",
    "Rishabam",
    "Mithunam",
    "Kadagam",
    "Simmam",
    "Kanni",
    "Thulam",
    "Viruchigam",
    "Dhanusu",
    "Magaram",
    "Kumbam",
    "Meenam",
];

#[derive(Debug, Error)]
pub enum HoroscopeError {
    #[error(transparent)]
    Panchang(#[from] panchang_core::PanchangError),
    #[error("dasha_horizon_years must be >= 0")]
    InvalidHorizon,
    #[error("horizon_end_jd_ut must be >= birth_jd_ut")]
    InvalidWindow,
}

#[derive(Debug, Deserialize)]
pub struct NatalChartRequest {
    pub birth_local: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
    #[serde(default)]
    pub dasha_horizon_years: Option<i32>,
    pub as_of_local: Option<String>,
}

/// Add calendar years to local civil datetime, clamping Feb 29 → Feb 28 when needed.
fn naive_add_calendar_years(dt: NaiveDateTime, years: i32) -> NaiveDateTime {
    let d = dt.date();
    let target_year = d.year() + years;
    let adjusted = NaiveDate::from_ymd_opt(target_year, d.month(), d.day()).unwrap_or_else(|| {
        NaiveDate::from_ymd_opt(target_year, 2, 28).expect("feb 28 exists")
    });
    adjusted.and_time(dt.time())
}

fn sidereal_body_core(lon_deg: f64) -> (u8, String, String, u8, String, String, u8) {
    let lon = reduce_deg(lon_deg);
    let rashi_index = (lon / 30.0).floor() as u8 + 1;
    let rashi_index = rashi_index.min(12).max(1);
    let span = 360.0 / 27.0;
    let nakshatra_index = (lon / span).floor() as u8 + 1;
    let nakshatra_index = nakshatra_index.min(27).max(1);
    let pos_in_nak = lon - (nakshatra_index as f64 - 1.0) * span;
    let nakshatra_pada = (pos_in_nak / (span / 4.0)).floor() as u8 + 1;
    let nakshatra_pada = nakshatra_pada.min(4).max(1);

    let rashi_name = names::RASHI_NAMES[(rashi_index - 1) as usize].to_string();
    let rashi_name_tamil = RASHI_NAMES_TAMIL[(rashi_index - 1) as usize].to_string();
    let nakshatra_name = names::NAKSHATRA_NAMES[(nakshatra_index - 1) as usize].to_string();
    let nakshatra_name_tamil =
        names::NAKSHATRA_NAMES_TAMIL[(nakshatra_index - 1) as usize].to_string();

    (
        rashi_index,
        rashi_name,
        rashi_name_tamil,
        nakshatra_index,
        nakshatra_name,
        nakshatra_name_tamil,
        nakshatra_pada,
    )
}

fn body_longitude_out(lon_deg: f64) -> BodyLongitudeOut {
    let (
        rashi_index,
        rashi_name,
        rashi_name_tamil,
        nakshatra_index,
        nakshatra_name,
        nakshatra_name_tamil,
        nakshatra_pada,
    ) = sidereal_body_core(lon_deg);
    BodyLongitudeOut {
        sidereal_longitude_deg: reduce_deg(lon_deg),
        rashi_index,
        rashi_name,
        rashi_name_tamil,
        nakshatra_index,
        nakshatra_name,
        nakshatra_name_tamil,
        nakshatra_pada,
    }
}

fn graha_full(lon_deg: f64, retrograde: bool) -> GrahaFullOut {
    let (rashi_index, rashi_name, _, nakshatra_index, nakshatra_name, _, nakshatra_pada) =
        sidereal_body_core(lon_deg);
    GrahaFullOut {
        sidereal_longitude_deg: reduce_deg(lon_deg),
        rashi_index,
        rashi_name,
        nakshatra_index,
        nakshatra_name,
        nakshatra_pada,
        retrograde,
    }
}

fn sidereal_slow_planet(jd: f64, ay_deg: f64, p: KeplerPlanet) -> GrahaFullOut {
    let trop = planet_geocentric_apparent_longitude_deg(jd, p);
    let lon = reduce_deg(trop - ay_deg);
    let retro = planet_geocentric_longitude_retrograde(jd, p, 0.08);
    graha_full(lon, retro)
}

/// Mean lunar north node (Rahu) + Ketu = Rahu + 180°; nodes flagged retrograde (mean motion).
fn sidereal_rahu_ketu(jd: f64, ay_deg: f64) -> (GrahaFullOut, GrahaFullOut) {
    let trop_rahu = mean_north_node_apparent_longitude_deg(jd);
    let sid_rahu = reduce_deg(trop_rahu - ay_deg);
    let sid_ketu = reduce_deg(sid_rahu + 180.0);
    (graha_full(sid_rahu, true), graha_full(sid_ketu, true))
}

/// Compute [`SouthIndianNatalChart`] for MCP / HTTP callers.
pub fn calculate_south_indian_natal_chart(req: NatalChartRequest) -> Result<SouthIndianNatalChart, HoroscopeError> {
    validate_observer(req.latitude, req.longitude)?;
    let horizon_years = req.dasha_horizon_years.unwrap_or(20);
    if horizon_years < 0 {
        return Err(HoroscopeError::InvalidHorizon);
    }

    let tz: Tz = parse_timezone(&req.timezone)?;
    let birth_local = parse_local_datetime(&req.birth_local)?;
    let birth_utc = local_to_utc(birth_local, tz);
    let jd_birth = julian_day_ut(birth_utc);

    let snap = snapshot(SnapshotRequest {
        when_local: req.birth_local.clone(),
        timezone: req.timezone.clone(),
        latitude: req.latitude,
        longitude: req.longitude,
        ayanamsha: req.ayanamsha,
        engine: req.engine,
    })?;

    let as_of_naive = match req.as_of_local.as_deref() {
        Some(s) => parse_local_datetime(s)?,
        None => Utc::now().with_timezone(&tz).naive_local(),
    };
    let as_of_jd = julian_day_ut(local_to_utc(as_of_naive, tz));
    let horizon_naive = naive_add_calendar_years(as_of_naive, horizon_years);
    let horizon_jd = julian_day_ut(local_to_utc(horizon_naive, tz));
    if horizon_jd + 1e-9 < jd_birth {
        return Err(HoroscopeError::InvalidWindow);
    }

    let trop_asc = ascendant_tropical_deg(jd_birth, req.latitude, req.longitude);
    let lagna_lon = reduce_deg(trop_asc - snap.ayanamsha_deg);
    let lagna = body_longitude_out(lagna_lon);

    let ay = snap.ayanamsha_deg;
    let (rahu, ketu) = sidereal_rahu_ketu(jd_birth, ay);

    let grahas = GrahasOut {
        sun: graha_full(snap.sun_sidereal_deg, false),
        moon: graha_full(snap.moon_sidereal_deg, false),
        mars: sidereal_slow_planet(jd_birth, ay, KeplerPlanet::Mars),
        mercury: sidereal_slow_planet(jd_birth, ay, KeplerPlanet::Mercury),
        jupiter: sidereal_slow_planet(jd_birth, ay, KeplerPlanet::Jupiter),
        venus: sidereal_slow_planet(jd_birth, ay, KeplerPlanet::Venus),
        saturn: sidereal_slow_planet(jd_birth, ay, KeplerPlanet::Saturn),
        rahu,
        ketu,
    };

    let birth_date = birth_local.date();
    let day = panchang_day(PanchangDayRequest {
        date: birth_date.format("%Y-%m-%d").to_string(),
        timezone: req.timezone.clone(),
        latitude: req.latitude,
        longitude: req.longitude,
        day_mode: Some(PanchangDayMode::CivilMidnight),
        ayanamsha: req.ayanamsha,
        engine: req.engine,
    })?;

    let panchang_at_birth = PanchangAtBirthOut {
        vaara: snap.vaara_civil_local.clone(),
        tithi_name: snap.angas.tithi_name.clone(),
        yoga_name: snap.angas.yoga_name.clone(),
        karana_name: snap.angas.karana_name.clone(),
        paksha: snap.angas.paksha.clone(),
        sunrise_local: snap.sunrise_jd_ut.map(|j| local_iso_from_jd(j, tz)),
        sunset_local: snap.sunset_jd_ut.map(|j| local_iso_from_jd(j, tz)),
    };

    let tamil_calendar_hint = TamilCalendarHintOut {
        solar_month_name: day.tamil_calendar.solar_month_name.clone(),
        solar_month_name_tamil: day.tamil_calendar.solar_month_name_tamil.clone(),
        tamil_year_name: day.tamil_calendar.tamil_year_name.clone(),
        weekday_name_tamil: day.tamil_calendar.weekday_name_tamil.clone(),
    };

    let dasha_bhukti = build_dasha_bhukti(
        jd_birth,
        snap.moon_sidereal_deg,
        snap.angas.nakshatra_index,
        as_of_jd,
        horizon_jd,
        horizon_years,
        &req.timezone,
        tz,
        as_of_naive.format("%Y-%m-%dT%H:%M:%S").to_string(),
        horizon_naive.format("%Y-%m-%dT%H:%M:%S").to_string(),
    );

    Ok(SouthIndianNatalChart {
        schema_version: SCHEMA_VERSION,
        kind: "south_indian_natal_chart",
        birth: BirthOut {
            birth_local: req.birth_local.clone(),
            timezone: req.timezone.clone(),
            latitude: req.latitude,
            longitude: req.longitude,
            utc_iso: birth_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            jd_ut: jd_birth,
        },
        frame: FrameOut {
            ayanamsha: snap.ayanamsha,
            ayanamsha_deg: snap.ayanamsha_deg,
            engine: snap.engine,
            sidereal_zodiac: "tropical_minus_ayanamsha",
            lunar_node_policy: "mean_ascending_plus_nutation",
            slow_planet_ephemeris: "jpl_kepler_table1_1800_2050",
        },
        lagna,
        grahas,
        panchang_at_birth,
        tamil_calendar_hint,
        dasha_bhukti,
        extensions: ExtensionsOut {
            navamsa_d9: serde_json::Value::Null,
            pratyantardasha: serde_json::Value::Null,
            notes: "Reserved for D9, Sookshma–Pratyantar, Ashtakavarga, etc.",
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_schema_and_lagna_reasonable() {
        let chart = calculate_south_indian_natal_chart(NatalChartRequest {
            birth_local: "1990-08-15T14:32:00".to_string(),
            timezone: "Asia/Kolkata".to_string(),
            latitude: 13.0827,
            longitude: 80.2707,
            ayanamsha: Some(AyanamshaId::Lahiri),
            engine: Some(EngineId::Meeus),
            dasha_horizon_years: Some(5),
            as_of_local: Some("2026-05-02T12:00:00".to_string()),
        })
        .expect("chart");

        assert_eq!(chart.schema_version, SCHEMA_VERSION);
        assert_eq!(chart.kind, "south_indian_natal_chart");
        assert!(chart.lagna.sidereal_longitude_deg >= 0.0 && chart.lagna.sidereal_longitude_deg < 360.0);
        assert!(!chart.dasha_bhukti.mahadashas.is_empty());
        assert_eq!(
            chart.grahas.moon.nakshatra_index,
            chart.dasha_bhukti.moon_at_birth.nakshatra_index
        );
        assert_eq!(chart.frame.lunar_node_policy, "mean_ascending_plus_nutation");
        let sep = reduce_deg(
            chart.grahas.ketu.sidereal_longitude_deg - chart.grahas.rahu.sidereal_longitude_deg,
        );
        assert!(
            (sep - 180.0).abs() < 0.02,
            "ketu should oppose rahu sidereally, sep={sep}"
        );
    }

    #[test]
    fn lagna_changes_with_sidereal_time() {
        let c1 = calculate_south_indian_natal_chart(NatalChartRequest {
            birth_local: "1990-08-15T14:32:00".to_string(),
            timezone: "Asia/Kolkata".to_string(),
            latitude: 13.0827,
            longitude: 80.2707,
            ayanamsha: None,
            engine: None,
            dasha_horizon_years: Some(1),
            as_of_local: Some("2026-01-01T00:00:00".to_string()),
        })
        .unwrap();
        let c2 = calculate_south_indian_natal_chart(NatalChartRequest {
            birth_local: "1990-08-15T20:32:00".to_string(),
            timezone: "Asia/Kolkata".to_string(),
            latitude: 13.0827,
            longitude: 80.2707,
            ayanamsha: None,
            engine: None,
            dasha_horizon_years: Some(1),
            as_of_local: Some("2026-01-01T00:00:00".to_string()),
        })
        .unwrap();
        assert!(
            (c1.lagna.sidereal_longitude_deg - c2.lagna.sidereal_longitude_deg).abs() > 1.0,
            "lagna should move over hours"
        );
    }

    #[test]
    fn navagraha_longitudes_present() {
        let chart = calculate_south_indian_natal_chart(NatalChartRequest {
            birth_local: "1999-06-21T12:00:00".to_string(),
            timezone: "Asia/Kolkata".to_string(),
            latitude: 13.0827,
            longitude: 80.2707,
            ayanamsha: Some(AyanamshaId::Lahiri),
            engine: Some(EngineId::Meeus),
            dasha_horizon_years: Some(1),
            as_of_local: Some("2026-01-01T00:00:00".to_string()),
        })
        .unwrap();
        let g = &chart.grahas;
        for lon in [
            g.mars.sidereal_longitude_deg,
            g.mercury.sidereal_longitude_deg,
            g.jupiter.sidereal_longitude_deg,
            g.venus.sidereal_longitude_deg,
            g.saturn.sidereal_longitude_deg,
            g.rahu.sidereal_longitude_deg,
            g.ketu.sidereal_longitude_deg,
        ] {
            assert!((0.0..360.0).contains(&lon), "lon={lon}");
        }
    }
}
