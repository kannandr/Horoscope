//! Serializable `south_indian_natal_chart` payload (schema_version 1.2.0).

use panchang_core::{AyanamshaId, EngineId};
use serde::Serialize;

pub const SCHEMA_VERSION: &str = "1.2.0";

#[derive(Debug, Serialize)]
pub struct SouthIndianNatalChart {
    pub schema_version: &'static str,
    pub kind: &'static str,
    pub birth: BirthOut,
    pub frame: FrameOut,
    pub lagna: BodyLongitudeOut,
    pub grahas: GrahasOut,
    pub panchang_at_birth: PanchangAtBirthOut,
    pub tamil_calendar_hint: TamilCalendarHintOut,
    pub dasha_bhukti: DashaBhuktiOut,
    pub extensions: ExtensionsOut,
}

#[derive(Debug, Serialize)]
pub struct BirthOut {
    pub birth_local: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub utc_iso: String,
    pub jd_ut: f64,
}

#[derive(Debug, Serialize)]
pub struct FrameOut {
    pub ayanamsha: AyanamshaId,
    pub ayanamsha_deg: f64,
    pub engine: EngineId,
    pub sidereal_zodiac: &'static str,
    /// `mean_lunar_node` — Rahu/Ketu use Meeus mean ascending node + nutation; tropical before ayanamsha.
    pub lunar_node_policy: &'static str,
    /// JPL Table‑1 Kepler fit (~1800–2050 AD); nutation applied to match Sun/Moon apparent longitude.
    pub slow_planet_ephemeris: &'static str,
}

#[derive(Debug, Serialize)]
pub struct BodyLongitudeOut {
    pub sidereal_longitude_deg: f64,
    pub rashi_index: u8,
    pub rashi_name: String,
    pub rashi_name_tamil: String,
    pub nakshatra_index: u8,
    pub nakshatra_name: String,
    pub nakshatra_name_tamil: String,
    pub nakshatra_pada: u8,
}

#[derive(Debug, Serialize)]
pub struct GrahaFullOut {
    pub sidereal_longitude_deg: f64,
    pub rashi_index: u8,
    pub rashi_name: String,
    pub nakshatra_index: u8,
    pub nakshatra_name: String,
    pub nakshatra_pada: u8,
    pub retrograde: bool,
}

#[derive(Debug, Serialize)]
pub struct GrahasOut {
    pub sun: GrahaFullOut,
    pub moon: GrahaFullOut,
    pub mars: GrahaFullOut,
    pub mercury: GrahaFullOut,
    pub jupiter: GrahaFullOut,
    pub venus: GrahaFullOut,
    pub saturn: GrahaFullOut,
    pub rahu: GrahaFullOut,
    pub ketu: GrahaFullOut,
}

#[derive(Debug, Serialize)]
pub struct PanchangAtBirthOut {
    pub vaara: String,
    pub tithi_name: String,
    pub yoga_name: String,
    pub karana_name: String,
    pub paksha: String,
    pub sunrise_local: Option<String>,
    pub sunset_local: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TamilCalendarHintOut {
    pub solar_month_name: String,
    pub solar_month_name_tamil: String,
    pub tamil_year_name: String,
    pub weekday_name_tamil: String,
}

#[derive(Debug, Serialize)]
pub struct DashaAliasesOut {
    pub bhukti_en: &'static str,
    pub bhukti_ta_hint: &'static str,
}

#[derive(Debug, Serialize)]
pub struct MahadashaYearsOut {
    pub ketu: u8,
    pub venus: u8,
    pub sun: u8,
    pub moon: u8,
    pub mars: u8,
    pub rahu: u8,
    pub jupiter: u8,
    pub saturn: u8,
    pub mercury: u8,
}

#[derive(Debug, Serialize)]
pub struct MoonAtBirthDashaOut {
    pub nakshatra_index: u8,
    pub nakshatra_name: String,
    pub starting_mahadasha_lord: String,
    pub balance_of_starting_mahadasha_at_birth_days: f64,
}

#[derive(Debug, Serialize)]
pub struct DashaWindowOut {
    pub birth_jd_ut: f64,
    pub as_of_local: String,
    pub as_of_jd_ut: f64,
    pub horizon_end_local: String,
    pub horizon_end_jd_ut: f64,
    pub horizon_years_after_as_of: i32,
    pub timezone: String,
}

#[derive(Debug, Serialize)]
pub struct AntardashaOut {
    pub lord: String,
    pub lord_display_en: String,
    pub lord_display_ta: &'static str,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    pub start_local: String,
    pub end_local: String,
}

#[derive(Debug, Serialize)]
pub struct MahadashaSegmentOut {
    pub lord: String,
    pub lord_display_en: String,
    pub lord_display_ta: &'static str,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    pub start_local: String,
    pub end_local: String,
    pub antardashas: Vec<AntardashaOut>,
}

#[derive(Debug, Serialize)]
pub struct DashaBhuktiOut {
    pub system: &'static str,
    pub aliases: DashaAliasesOut,
    pub lords_order: [&'static str; 9],
    pub mahadasha_years: MahadashaYearsOut,
    pub moon_at_birth: MoonAtBirthDashaOut,
    pub window: DashaWindowOut,
    pub mahadashas: Vec<MahadashaSegmentOut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ExtensionsOut {
    pub navamsa_d9: serde_json::Value,
    pub pratyantardasha: serde_json::Value,
    pub notes: &'static str,
}
