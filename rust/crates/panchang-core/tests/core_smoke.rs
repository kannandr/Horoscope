use panchang_core::{
    civil_day, month, panchang_day, search_muhurta, snapshot, CivilDayRequest, MonthRequest,
    MuhurtaSearchRequest, PanchangDayMode, PanchangDayRequest, SnapshotRequest,
};

#[test]
fn snapshot_bangalore_smoke() {
    let out = snapshot(SnapshotRequest {
        when_local: "2026-04-30T12:00:00".to_string(),
        timezone: "Asia/Kolkata".to_string(),
        latitude: 12.97,
        longitude: 77.59,
        ayanamsha: None,
        engine: None,
    })
    .expect("snapshot");
    assert!((1..=30).contains(&out.angas.tithi_index));
    assert!((1..=27).contains(&out.angas.nakshatra_index));
    assert!(out.jd_ut > 2460000.0);
}

#[test]
fn civil_day_has_segments() {
    let out = civil_day(CivilDayRequest {
        date: "2026-04-30".to_string(),
        timezone: "Asia/Kolkata".to_string(),
        latitude: 12.97,
        longitude: 77.59,
        ayanamsha: None,
        engine: None,
    })
    .expect("civil day");
    assert!(!out.tithi_intervals.is_empty());
    assert!(!out.nakshatra_intervals.is_empty());
    assert!(!out.yoga_intervals.is_empty());
    assert!(!out.karana_intervals.is_empty());
    assert!(out
        .tithi_intervals
        .iter()
        .all(|x| x.clipped_start_jd_ut >= x.start_jd_ut && x.clipped_end_jd_ut <= x.end_jd_ut));
}

#[test]
fn panchang_day_has_hora_and_bad_periods() {
    let out = panchang_day(PanchangDayRequest {
        date: "2026-04-30".to_string(),
        timezone: "Asia/Kolkata".to_string(),
        latitude: 12.97,
        longitude: 77.59,
        day_mode: Some(PanchangDayMode::SunriseDay),
        ayanamsha: None,
        engine: None,
    })
    .expect("panchang day");
    assert_eq!(out.day_mode, PanchangDayMode::SunriseDay);
    assert!(out.day_start_jd_ut < out.day_end_jd_ut);
    assert_eq!(out.hora.len(), 24);
    assert!(!out.tithi_intervals.is_empty());
    assert!(!out.nakshatra_intervals.is_empty());
    assert!(!out.yoga_intervals.is_empty());
    assert!(!out.karana_intervals.is_empty());
    assert_eq!(out.inauspicious_periods.len(), 3);
    assert_eq!(out.auspicious_periods.len(), 1);
    assert_eq!(out.tamil_calendar.solar_month_name, "Chithirai");
    assert_eq!(out.tamil_calendar.tamil_year_name, "Parabhava");
    assert!(out
        .inauspicious_periods
        .iter()
        .all(|p| p.jd_start < p.jd_end));
}

#[test]
fn month_and_muhurta_run() {
    let out = month(MonthRequest {
        year: 2026,
        month: 4,
        timezone: "America/Los_Angeles".to_string(),
        latitude: 37.6819,
        longitude: -121.768,
        ayanamsha: None,
        engine: None,
    })
    .expect("month");
    assert_eq!(out.days.len(), 30);

    let muhurta = search_muhurta(MuhurtaSearchRequest {
        date_start: "2026-04-30".to_string(),
        date_end: "2026-04-30".to_string(),
        timezone: "Asia/Kolkata".to_string(),
        latitude: 12.97,
        longitude: 77.59,
        purpose_preset: None,
        min_duration_minutes: Some(45),
        ayanamsha: None,
        engine: None,
    })
    .expect("muhurta");
    assert_eq!(muhurta.preset, "south_indian_tamil_general");
}
