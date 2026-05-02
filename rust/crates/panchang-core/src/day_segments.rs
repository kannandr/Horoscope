use chrono::{Datelike, Duration, NaiveDate};
use chrono_tz::Tz;

use crate::types::{
    validate_observer, AyanamshaId, CivilDayRequest, CivilDayResponse, DayPeriod, EngineId,
    MonthDay, MonthRequest, MonthResponse, PanchangDayMode, PanchangDayRequest,
    PanchangDayResponse, PanchangError, Segment, TamilCalendarDay,
};
use crate::{angas, ayanamsha, boundaries, ephemeris, hora, names, rise_set, time};

fn angas_at_jd(jd: f64, ay: AyanamshaId, engine: EngineId) -> crate::types::PanchangAngas {
    let trop = ephemeris::apparent_tropical_longitudes(jd, engine);
    let d = ayanamsha::delta_deg(jd, ay);
    angas::compute_angas(
        ephemeris::reduce_deg(trop.sun_deg - d),
        ephemeris::reduce_deg(trop.moon_lon_deg - d),
    )
}

#[derive(Clone, Copy)]
enum AngaIntervalKind {
    Tithi,
    Nakshatra,
    Yoga,
    Karana,
}

impl AngaIntervalKind {
    fn max_segments(self) -> usize {
        match self {
            Self::Tithi => 4,
            Self::Nakshatra => 5,
            Self::Yoga => 5,
            Self::Karana => 8,
        }
    }

    fn boundaries(self, jd: f64, ay: AyanamshaId, engine: EngineId) -> Option<(f64, f64)> {
        match self {
            Self::Tithi => Some((
                boundaries::prev_tithi_start_jd(jd, ay, engine)?,
                boundaries::next_tithi_end_jd(jd, ay, engine)?,
            )),
            Self::Nakshatra => Some((
                boundaries::prev_nakshatra_start_jd(jd, ay, engine)?,
                boundaries::next_nakshatra_end_jd(jd, ay, engine)?,
            )),
            Self::Yoga => Some((
                boundaries::prev_yoga_start_jd(jd, ay, engine)?,
                boundaries::next_yoga_end_jd(jd, ay, engine)?,
            )),
            Self::Karana => Some((
                boundaries::prev_karana_start_jd(jd, ay, engine)?,
                boundaries::next_karana_end_jd(jd, ay, engine)?,
            )),
        }
    }

    fn label(self, jd: f64, ay: AyanamshaId, engine: EngineId) -> (String, Option<u8>) {
        let pa = angas_at_jd(jd, ay, engine);
        match self {
            Self::Tithi => (pa.tithi_name, None),
            Self::Nakshatra => (pa.nakshatra_name, Some(pa.nakshatra_pada)),
            Self::Yoga => (pa.yoga_name, None),
            Self::Karana => (pa.karana_name, None),
        }
    }
}

fn make_segment(
    name: String,
    start: f64,
    end: f64,
    window_start: f64,
    window_end: f64,
    tz: Tz,
    pada: Option<u8>,
) -> Segment {
    let clipped_start = start.max(window_start);
    let clipped_end = end.min(window_end);
    Segment {
        name,
        start_jd_ut: start,
        end_jd_ut: end,
        start_local: time::local_iso_from_jd(start, tz),
        end_local: time::local_iso_from_jd(end, tz),
        clipped_start_jd_ut: clipped_start,
        clipped_end_jd_ut: clipped_end,
        clipped_start_local: time::local_iso_from_jd(clipped_start, tz),
        clipped_end_local: time::local_iso_from_jd(clipped_end, tz),
        starts_before_window: start < window_start - 1e-9,
        ends_after_window: end > window_end + 1e-9,
        pada,
    }
}

fn intervals_between(
    window_start: f64,
    window_end: f64,
    tz: Tz,
    ay: AyanamshaId,
    engine: EngineId,
    kind: AngaIntervalKind,
) -> Vec<Segment> {
    let mut cur = window_start + 1e-7;
    let mut out = Vec::new();
    for _ in 0..kind.max_segments() {
        if cur >= window_end {
            break;
        }
        let Some((start, end)) = kind.boundaries(cur, ay, engine) else {
            break;
        };
        if end <= cur {
            break;
        }
        let probe = cur.max(start + 1e-7).min(end - 1e-7);
        let (name, pada) = kind.label(probe, ay, engine);
        out.push(make_segment(
            name,
            start,
            end,
            window_start,
            window_end,
            tz,
            pada,
        ));
        if end >= window_end - 1e-12 {
            break;
        }
        cur = end + 1e-7;
    }
    out
}

fn civil_window(date: NaiveDate, tz: Tz) -> (f64, f64) {
    let jd0 = time::jd_from_local_midnight(date, tz);
    let jd1 = time::jd_from_local_midnight(date + Duration::days(1), tz);
    (jd0, jd1)
}

fn tithi_intervals(date: NaiveDate, tz: Tz, ay: AyanamshaId, engine: EngineId) -> Vec<Segment> {
    let (jd0, jd1) = civil_window(date, tz);
    intervals_between(jd0, jd1, tz, ay, engine, AngaIntervalKind::Tithi)
}

fn nakshatra_intervals(date: NaiveDate, tz: Tz, ay: AyanamshaId, engine: EngineId) -> Vec<Segment> {
    let (jd0, jd1) = civil_window(date, tz);
    intervals_between(jd0, jd1, tz, ay, engine, AngaIntervalKind::Nakshatra)
}

fn yoga_intervals(date: NaiveDate, tz: Tz, ay: AyanamshaId, engine: EngineId) -> Vec<Segment> {
    let (jd0, jd1) = civil_window(date, tz);
    intervals_between(jd0, jd1, tz, ay, engine, AngaIntervalKind::Yoga)
}

fn karana_intervals(date: NaiveDate, tz: Tz, ay: AyanamshaId, engine: EngineId) -> Vec<Segment> {
    let (jd0, jd1) = civil_window(date, tz);
    intervals_between(jd0, jd1, tz, ay, engine, AngaIntervalKind::Karana)
}

fn weekday_index_at_jd(jd: f64, tz: Tz) -> usize {
    time::datetime_utc_from_jd(jd)
        .with_timezone(&tz)
        .weekday()
        .num_days_from_monday() as usize
}

fn weekday_name_at_jd(jd: f64, tz: Tz) -> String {
    names::weekday_name(weekday_index_at_jd(jd, tz)).to_string()
}

fn day_period(
    code: &str,
    name: &str,
    category: &str,
    start: f64,
    end: f64,
    tz: Tz,
    source: &str,
) -> DayPeriod {
    DayPeriod {
        code: code.to_string(),
        name: name.to_string(),
        category: category.to_string(),
        jd_start: start,
        jd_end: end,
        start_local: time::local_iso_from_jd(start, tz),
        end_local: time::local_iso_from_jd(end, tz),
        source: source.to_string(),
    }
}

fn eighth_period(sr: f64, ss: f64, eighth_index: usize) -> (f64, f64) {
    let len = (ss - sr) / 8.0;
    let start = sr + len * eighth_index as f64;
    (start, start + len)
}

fn inauspicious_periods(sr: f64, ss: f64, weekday_index: usize, tz: Tz) -> Vec<DayPeriod> {
    const RAHU: [usize; 7] = [1, 6, 4, 5, 3, 2, 7];
    const YAMA: [usize; 7] = [3, 2, 1, 0, 6, 5, 4];
    const GULIKA: [usize; 7] = [5, 4, 3, 2, 1, 0, 6];

    [
        ("rahu_kalam", "Rahu Kalam", RAHU[weekday_index]),
        ("yama_gandam", "Yama Gandam", YAMA[weekday_index]),
        ("gulika_kalam", "Gulika Kalam", GULIKA[weekday_index]),
    ]
    .into_iter()
    .map(|(code, name, idx)| {
        let (start, end) = eighth_period(sr, ss, idx);
        day_period(
            code,
            name,
            "inauspicious",
            start,
            end,
            tz,
            "south_indian_day_eighths_v1",
        )
    })
    .collect()
}

fn auspicious_periods(sr: f64, ss: f64, tz: Tz) -> Vec<DayPeriod> {
    let day_length = ss - sr;
    let length = day_length / 15.0;
    let noon = (sr + ss) / 2.0;
    vec![day_period(
        "abhijit_muhurta",
        "Abhijit Muhurta",
        "auspicious",
        noon - length / 2.0,
        noon + length / 2.0,
        tz,
        "day_fifteenths_centered_on_local_noon_v1",
    )]
}

fn tamil_year_index(date: NaiveDate, sun_rashi_index: u8) -> u8 {
    let solar_year = if date.month() < 4 || (date.month() == 4 && sun_rashi_index == 12) {
        date.year() - 1
    } else {
        date.year()
    };
    (solar_year + 53).rem_euclid(60) as u8
}

fn ayana_name(sun_rashi_index: u8) -> &'static str {
    match sun_rashi_index {
        10..=12 | 1..=3 => "Uttarayana",
        _ => "Dakshinayana",
    }
}

fn ritu_name(sun_rashi_index: u8) -> &'static str {
    match sun_rashi_index {
        1 | 2 => "Vasanta",
        3 | 4 => "Grishma",
        5 | 6 => "Varsha",
        7 | 8 => "Sharad",
        9 | 10 => "Hemanta",
        _ => "Shishira",
    }
}

fn tamil_calendar_day(
    date: NaiveDate,
    jd: f64,
    tz: Tz,
    ay: AyanamshaId,
    engine: EngineId,
) -> TamilCalendarDay {
    let pa = angas_at_jd(jd, ay, engine);
    let month_idx = pa.sun_rashi_index;
    let year_idx = tamil_year_index(date, month_idx);
    TamilCalendarDay {
        solar_month_index: month_idx,
        solar_month_name: names::TAMIL_SOLAR_MONTH_NAMES[(month_idx - 1) as usize].to_string(),
        solar_month_name_tamil: names::TAMIL_SOLAR_MONTH_NAMES_TAMIL[(month_idx - 1) as usize]
            .to_string(),
        tamil_year_index: year_idx + 1,
        tamil_year_name: names::TAMIL_YEAR_NAMES[year_idx as usize].to_string(),
        ayana: ayana_name(month_idx).to_string(),
        ritu: ritu_name(month_idx).to_string(),
        weekday_name_tamil: names::WEEKDAY_NAMES_TAMIL[weekday_index_at_jd(jd, tz)].to_string(),
    }
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
        yoga_intervals: yoga_intervals(date, tz, ay, engine),
        karana_intervals: karana_intervals(date, tz, ay, engine),
    })
}

pub fn panchang_day(req: PanchangDayRequest) -> Result<PanchangDayResponse, PanchangError> {
    validate_observer(req.latitude, req.longitude)?;
    let tz = time::parse_timezone(&req.timezone)?;
    let date = time::parse_date(&req.date)?;
    let ay = req.ayanamsha.unwrap_or_default();
    let engine = req.engine.unwrap_or_default();
    let mode = req.day_mode.unwrap_or_default();
    let (civil_start, civil_end) = civil_window(date, tz);

    let sunrise_jd_ut =
        rise_set::find_next_sunrise_jd(civil_start - 10.0 / 1440.0, req.latitude, req.longitude);
    let sunset_jd_ut = sunrise_jd_ut
        .and_then(|sr| rise_set::find_next_sunset_jd(sr + 1e-6, req.latitude, req.longitude));
    let next_sunrise_jd_ut = sunset_jd_ut
        .and_then(|ss| rise_set::find_next_sunrise_jd(ss + 1e-6, req.latitude, req.longitude));

    let (day_start, day_end) = match mode {
        PanchangDayMode::CivilMidnight => (civil_start, civil_end),
        PanchangDayMode::SunriseDay => {
            let sr = sunrise_jd_ut.ok_or_else(|| {
                PanchangError::Calculation(
                    "sunrise was not found for this date/location".to_string(),
                )
            })?;
            let ns = next_sunrise_jd_ut.ok_or_else(|| {
                PanchangError::Calculation(
                    "next sunrise was not found for this date/location".to_string(),
                )
            })?;
            (sr, ns)
        }
    };

    let hora = match (sunrise_jd_ut, sunset_jd_ut, next_sunrise_jd_ut) {
        (Some(sr), Some(ss), Some(ns)) => hora::build_hora_table(sr, ss, ns, tz),
        _ => Vec::new(),
    };
    let vaara_at_sunrise = sunrise_jd_ut.map(|sr| weekday_name_at_jd(sr, tz));
    let angas_at_sunrise = sunrise_jd_ut.map(|sr| angas_at_jd(sr + 1e-7, ay, engine));
    let tamil_calendar = tamil_calendar_day(
        date,
        sunrise_jd_ut.unwrap_or(day_start) + 1e-7,
        tz,
        ay,
        engine,
    );
    let (inauspicious_periods, auspicious_periods) = match (sunrise_jd_ut, sunset_jd_ut) {
        (Some(sr), Some(ss)) => {
            let weekday_index = weekday_index_at_jd(sr, tz);
            (
                inauspicious_periods(sr, ss, weekday_index, tz),
                auspicious_periods(sr, ss, tz),
            )
        }
        _ => (Vec::new(), Vec::new()),
    };

    Ok(PanchangDayResponse {
        date: req.date,
        timezone: req.timezone,
        day_mode: mode,
        day_start_jd_ut: day_start,
        day_end_jd_ut: day_end,
        day_start_local: time::local_iso_from_jd(day_start, tz),
        day_end_local: time::local_iso_from_jd(day_end, tz),
        sunrise_jd_ut,
        sunrise_local: sunrise_jd_ut.map(|x| time::local_iso_from_jd(x, tz)),
        sunset_jd_ut,
        sunset_local: sunset_jd_ut.map(|x| time::local_iso_from_jd(x, tz)),
        next_sunrise_jd_ut,
        next_sunrise_local: next_sunrise_jd_ut.map(|x| time::local_iso_from_jd(x, tz)),
        vaara_civil_local: names::weekday_name(date.weekday().num_days_from_monday() as usize)
            .to_string(),
        vaara_at_sunrise,
        angas_at_sunrise,
        tamil_calendar,
        tithi_intervals: intervals_between(
            day_start,
            day_end,
            tz,
            ay,
            engine,
            AngaIntervalKind::Tithi,
        ),
        nakshatra_intervals: intervals_between(
            day_start,
            day_end,
            tz,
            ay,
            engine,
            AngaIntervalKind::Nakshatra,
        ),
        yoga_intervals: intervals_between(
            day_start,
            day_end,
            tz,
            ay,
            engine,
            AngaIntervalKind::Yoga,
        ),
        karana_intervals: intervals_between(
            day_start,
            day_end,
            tz,
            ay,
            engine,
            AngaIntervalKind::Karana,
        ),
        hora,
        inauspicious_periods,
        auspicious_periods,
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
        let yoga_intervals = yoga_intervals(date, tz, ay, engine);
        let karana_intervals = karana_intervals(date, tz, ay, engine);
        days.push(MonthDay {
            date: date.to_string(),
            tithi_leader: tithi_intervals.first().map(|x| x.name.clone()),
            nakshatra_leader: nakshatra_intervals.first().map(|x| x.name.clone()),
            yoga_leader: yoga_intervals.first().map(|x| x.name.clone()),
            karana_leader: karana_intervals.first().map(|x| x.name.clone()),
            tithi_intervals,
            nakshatra_intervals,
            yoga_intervals,
            karana_intervals,
        });
        date += Duration::days(1);
    }
    Ok(MonthResponse {
        year: req.year,
        month: req.month,
        timezone: req.timezone,
        days,
    })
}
