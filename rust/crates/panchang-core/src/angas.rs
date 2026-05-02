use crate::names;
use crate::types::PanchangAngas;

pub fn elongation_deg(sun_sidereal: f64, moon_sidereal: f64) -> f64 {
    (moon_sidereal - sun_sidereal).rem_euclid(360.0)
}

pub fn paksha_and_day(elong: f64) -> (&'static str, u8) {
    if elong < 180.0 {
        ("shukla", (elong / 12.0).floor() as u8 + 1)
    } else {
        ("krishna", ((elong - 180.0) / 12.0).floor() as u8 + 1)
    }
}

fn describe_tithi_name(paksha: &str, day: u8) -> String {
    if day == 15 {
        if paksha == "shukla" { "Purnima".to_string() } else { "Amavasya".to_string() }
    } else {
        names::TITHI_NAMES[(day - 1) as usize].to_string()
    }
}

fn karana_name_from_half_index(half: u8) -> &'static str {
    match half % 60 {
        0 => "Kimstughna",
        57 => "Shakuni",
        58 => "Chatushpada",
        59 => "Naga",
        h => ["Bava", "Balava", "Kaulava", "Taitila", "Gara", "Vanija", "Vishti"][((h - 1) % 7) as usize],
    }
}

pub fn compute_angas(sun_sidereal: f64, moon_sidereal: f64) -> PanchangAngas {
    let el = elongation_deg(sun_sidereal, moon_sidereal);
    let tithi_index = (el / 12.0).floor() as u8 + 1;
    let (paksha, paksha_day) = paksha_and_day(el);
    let span = 360.0 / 27.0;
    let nakshatra_index = (moon_sidereal / span).floor() as u8 + 1;
    let nakshatra_pada = ((moon_sidereal % span) / (span / 4.0)).floor() as u8 + 1;
    let yoga_index = (((sun_sidereal + moon_sidereal).rem_euclid(360.0)) / span).floor() as u8 + 1;
    let sun_rashi_index = ((sun_sidereal % 360.0) / 30.0).floor() as u8 + 1;
    let moon_rashi_index = ((moon_sidereal % 360.0) / 30.0).floor() as u8 + 1;
    PanchangAngas {
        elongation_deg: el,
        tithi_index,
        tithi_name: describe_tithi_name(paksha, paksha_day),
        paksha: paksha.to_string(),
        paksha_day,
        nakshatra_index,
        nakshatra_name: names::NAKSHATRA_NAMES[(nakshatra_index - 1) as usize].to_string(),
        nakshatra_name_tamil: names::NAKSHATRA_NAMES_TAMIL[(nakshatra_index - 1) as usize].to_string(),
        nakshatra_pada,
        yoga_index,
        yoga_name: names::YOGA_NAMES[(yoga_index - 1) as usize].to_string(),
        karana_name: karana_name_from_half_index((el / 6.0).floor() as u8).to_string(),
        sun_rashi_index,
        sun_rashi_name: names::RASHI_NAMES[(sun_rashi_index - 1) as usize].to_string(),
        moon_rashi_index,
        moon_rashi_name: names::RASHI_NAMES[(moon_rashi_index - 1) as usize].to_string(),
    }
}
