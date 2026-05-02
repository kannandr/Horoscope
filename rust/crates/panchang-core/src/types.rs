use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Error)]
pub enum PanchangError {
    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),
    #[error("invalid local datetime, expected RFC3339 or YYYY-MM-DDTHH:MM:SS: {0}")]
    InvalidDateTime(String),
    #[error("invalid date, expected YYYY-MM-DD: {0}")]
    InvalidDate(String),
    #[error("invalid observer coordinates")]
    InvalidCoordinates,
    #[error("calculation failed: {0}")]
    Calculation(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineId {
    Meeus,
    SuryaMean,
}

impl Default for EngineId {
    fn default() -> Self {
        Self::Meeus
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AyanamshaId {
    Lahiri,
    LahiriAltStub,
    Raman,
}

impl Default for AyanamshaId {
    fn default() -> Self {
        Self::Lahiri
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct SnapshotRequest {
    pub when_local: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CivilDayRequest {
    pub date: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PanchangDayMode {
    CivilMidnight,
    SunriseDay,
}

impl Default for PanchangDayMode {
    fn default() -> Self {
        Self::CivilMidnight
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct PanchangDayRequest {
    pub date: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub day_mode: Option<PanchangDayMode>,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MonthRequest {
    pub year: i32,
    pub month: u32,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MuhurtaSearchRequest {
    pub date_start: String,
    pub date_end: String,
    pub timezone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub purpose_preset: Option<String>,
    pub min_duration_minutes: Option<u32>,
    pub ayanamsha: Option<AyanamshaId>,
    pub engine: Option<EngineId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct PanchangAngas {
    pub elongation_deg: f64,
    pub tithi_index: u8,
    pub tithi_name: String,
    pub paksha: String,
    pub paksha_day: u8,
    pub nakshatra_index: u8,
    pub nakshatra_name: String,
    pub nakshatra_name_tamil: String,
    pub nakshatra_pada: u8,
    pub yoga_index: u8,
    pub yoga_name: String,
    pub karana_name: String,
    pub sun_rashi_index: u8,
    pub sun_rashi_name: String,
    pub moon_rashi_index: u8,
    pub moon_rashi_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct HoraInterval {
    pub index: u8,
    pub ruler: String,
    pub is_daytime: bool,
    pub jd_start: f64,
    pub jd_end: f64,
    pub start_local: String,
    pub end_local: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct SnapshotResponse {
    pub jd_ut: f64,
    pub engine: EngineId,
    pub ayanamsha: AyanamshaId,
    pub ayanamsha_deg: f64,
    pub sun_tropical_deg: f64,
    pub moon_tropical_deg: f64,
    pub moon_tropical_lat_deg: f64,
    pub sun_sidereal_deg: f64,
    pub moon_sidereal_deg: f64,
    pub angas: PanchangAngas,
    pub vaara_civil_local: String,
    pub sunrise_jd_ut: Option<f64>,
    pub sunset_jd_ut: Option<f64>,
    pub next_sunrise_jd_ut: Option<f64>,
    pub tithi_start_jd_ut: Option<f64>,
    pub next_tithi_end_jd: Option<f64>,
    pub next_nakshatra_end_jd: Option<f64>,
    pub next_yoga_end_jd: Option<f64>,
    pub karana_start_jd_ut: Option<f64>,
    pub karana_end_jd_ut: Option<f64>,
    pub karana_start_local: Option<String>,
    pub karana_end_local: Option<String>,
    pub hora: Vec<HoraInterval>,
    pub current_hora: Option<HoraInterval>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct Segment {
    pub name: String,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
    pub start_local: String,
    pub end_local: String,
    pub clipped_start_jd_ut: f64,
    pub clipped_end_jd_ut: f64,
    pub clipped_start_local: String,
    pub clipped_end_local: String,
    pub starts_before_window: bool,
    pub ends_after_window: bool,
    pub pada: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CivilDayResponse {
    pub date: String,
    pub timezone: String,
    pub tithi_intervals: Vec<Segment>,
    pub nakshatra_intervals: Vec<Segment>,
    pub yoga_intervals: Vec<Segment>,
    pub karana_intervals: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct DayPeriod {
    pub code: String,
    pub name: String,
    pub category: String,
    pub jd_start: f64,
    pub jd_end: f64,
    pub start_local: String,
    pub end_local: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct TamilCalendarDay {
    pub solar_month_index: u8,
    pub solar_month_name: String,
    pub solar_month_name_tamil: String,
    pub tamil_year_index: u8,
    pub tamil_year_name: String,
    pub ayana: String,
    pub ritu: String,
    pub weekday_name_tamil: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct PanchangDayResponse {
    pub date: String,
    pub timezone: String,
    pub day_mode: PanchangDayMode,
    pub day_start_jd_ut: f64,
    pub day_end_jd_ut: f64,
    pub day_start_local: String,
    pub day_end_local: String,
    pub sunrise_jd_ut: Option<f64>,
    pub sunrise_local: Option<String>,
    pub sunset_jd_ut: Option<f64>,
    pub sunset_local: Option<String>,
    pub next_sunrise_jd_ut: Option<f64>,
    pub next_sunrise_local: Option<String>,
    pub vaara_civil_local: String,
    pub vaara_at_sunrise: Option<String>,
    pub angas_at_sunrise: Option<PanchangAngas>,
    pub tamil_calendar: TamilCalendarDay,
    pub tithi_intervals: Vec<Segment>,
    pub nakshatra_intervals: Vec<Segment>,
    pub yoga_intervals: Vec<Segment>,
    pub karana_intervals: Vec<Segment>,
    pub hora: Vec<HoraInterval>,
    pub inauspicious_periods: Vec<DayPeriod>,
    pub auspicious_periods: Vec<DayPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MonthDay {
    pub date: String,
    pub tithi_leader: Option<String>,
    pub nakshatra_leader: Option<String>,
    pub yoga_leader: Option<String>,
    pub karana_leader: Option<String>,
    pub tithi_intervals: Vec<Segment>,
    pub nakshatra_intervals: Vec<Segment>,
    pub yoga_intervals: Vec<Segment>,
    pub karana_intervals: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MonthResponse {
    pub year: i32,
    pub month: u32,
    pub timezone: String,
    pub days: Vec<MonthDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MuhurtaWindow {
    pub start_local: String,
    pub end_local: String,
    pub duration_minutes: u32,
    pub score: i32,
    pub label: String,
    pub reasons: Vec<String>,
    pub exclusions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct MuhurtaSearchResponse {
    pub preset: String,
    pub timezone: String,
    pub windows: Vec<MuhurtaWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub request_id: Option<String>,
}

pub(crate) fn validate_observer(latitude: f64, longitude: f64) -> Result<(), PanchangError> {
    if (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude) {
        Ok(())
    } else {
        Err(PanchangError::InvalidCoordinates)
    }
}

