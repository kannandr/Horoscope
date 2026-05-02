use chrono::{Datelike, Duration, NaiveDate};

use crate::{angas, ayanamsha, boundaries, ephemeris, time};
use crate::types::{
    validate_observer, AyanamshaId, CivilDayRequest, CivilDayResponse, EngineId, MonthDay,
    MonthRequest, MonthResponse, PanchangError, Segment,
};

fn angas_at_jd(jd: f64, ay: AyanamshaId, engine: EngineId) -> crate::types::PanchangAngas {
    let trop = ephemeris::apparent_tropical_longitudes(jd, engine);
    let d = ayanamsha::delta_deg(jd, ay);
    angas::compute_angas(ephemeris::reduce_deg(trop.sun_deg - d), ephemeris::reduce_deg(trop.moon_lon_deg - d))
}

fn tithi_intervals(date: NaiveDate, tz: chrono_tz::Tz, ay: AyanamshaId, engine: EngineId) -> Vec<Segment> {
    let jd0 = time::jd_from_local_midnight(date, tz);
    let jd1 = time::jd_from_local_midnight(date + Duration::days(1), tz);
    let mut cur = jd0 + 1e-7;
    let mut out = Vec::new();
    for _ in 0..4 {
        if cur >= jd1 { break; }
        let Some(start) = boundaries::prev_tithi_start_jd(cur, ay, engine) else { break; };
        let Some(end) = boundaries::next_tithi_end_jd(cur, ay, engine) else { break; };
        let pa = angas_at_jd(cur, ay, engine);
        out.push(Segment {
            name: pa.tithi_name,
            start_jd_ut: start,
            end_jd_ut: end,
            start_local: time::local_iso_from_jd(start, tz),
            end_local: time::local_iso_from_jd(end, tz),
            pada: None,
        });
        if end >= jd1 - 1e-12 { break; }
        cur = end + 1e-7;
    }
    out
}

fn nakshatra_intervals(date: NaiveDate, tz: chrono_tz::Tz, ay: AyanamshaId, engine: EngineId) -> Vec<Segment> {
    let jd0 = time::jd_from_local_midnight(date, tz);
    let jd1 = time::jd_from_local_midnight(date + Duration::days(1), tz);
    let mut cur = jd0 + 1e-7;
    let mut out = Vec::new();
    for _ in 0..5 {
        if cur >= jd1 { break; }
        let Some(start) = boundaries::prev_nakshatra_start_jd(cur, ay, engine) else { break; };
        let Some(end) = boundaries::next_nakshatra_end_jd(cur, ay, engine) else { break; };
        let pa = angas_at_jd((start + (end - start) * 0.02).min(end - 1e-9), ay, engine);
        out.push(Segment {
            name: pa.nakshatra_name,
            start_jd_ut: start,
            end_jd_ut: end,
            start_local: time::local_iso_from_jd(start, tz),
            end_local: time::local_iso_from_jd(end, tz),
            pada: Some(pa.nakshatra_pada),
        });
        if end >= jd1 - 1e-12 { break; }
        cur = end + 1e-7;
    }
    out
}

pub fn civil_day(req: CivilDayRequest) -> Result<CivilDayResponse, PanchangError> {
    validate_observer(req.latitude, req.longitude)?;
    let tz = time::parse_timezone(&req.timezone)?;
    let date = time::parse_date(&req.date)?;
    let ay = req.ayanamsha.unwrap_or_default();
    let engine = req.engine.unwrap_or_default();
    Ok(CivilDayResponse {
        date: req.date,
        timezone: req.timezone,
        tithi_intervals: tithi_intervals(date, tz, ay, engine),
        nakshatra_intervals: nakshatra_intervals(date, tz, ay, engine),
    })
}

pub fn month(req: MonthRequest) -> Result<MonthResponse, PanchangError> {
    validate_observer(req.latitude, req.longitude)?;
    let tz = time::parse_timezone(&req.timezone)?;
    let ay = req.ayanamsha.unwrap_or_default();
    let engine = req.engine.unwrap_or_default();
    let first = NaiveDate::from_ymd_opt(req.year, req.month, 1)
        .ok_or_else(|| PanchangError::InvalidDate(format!("{}-{}", req.year, req.month)))?;
    let mut date = first;
    let mut days = Vec::new();
    while date.month() == req.month {
        let tithi_intervals = tithi_intervals(date, tz, ay, engine);
        let nakshatra_intervals = nakshatra_intervals(date, tz, ay, engine);
        days.push(MonthDay {
            date: date.to_string(),
            tithi_leader: tithi_intervals.first().map(|x| x.name.clone()),
            nakshatra_leader: nakshatra_intervals.first().map(|x| x.name.clone()),
            tithi_intervals,
            nakshatra_intervals,
        });
        date += Duration::days(1);
    }
    Ok(MonthResponse { year: req.year, month: req.month, timezone: req.timezone, days })
}
