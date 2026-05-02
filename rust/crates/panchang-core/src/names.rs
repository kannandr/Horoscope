pub const TITHI_NAMES: [&str; 15] = [
    "Pratipada",
    "Dvitiya",
    "Tritiya",
    "Chaturthi",
    "Panchami",
    "Shashthi",
    "Saptami",
    "Ashtami",
    "Navami",
    "Dashami",
    "Ekadashi",
    "Dvadashi",
    "Trayodashi",
    "Chaturdashi",
    "Purnima/Amavasya",
];

pub const NAKSHATRA_NAMES: [&str; 27] = [
    "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra", "Punarvasu", "Pushya",
    "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati",
    "Vishakha", "Anuradha", "Jyeshtha", "Mula", "Purva Ashadha", "Uttara Ashadha",
    "Shravana", "Dhanishta", "Shatabhisha", "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];

pub const NAKSHATRA_NAMES_TAMIL: [&str; 27] = [
    "Aswini", "Bharani", "Karthikai", "Rohini", "Mirugasirisham", "Thiruvathirai",
    "Punarpoosam", "Poosam", "Ayilyam", "Magam", "Pooram", "Uthiram", "Hastham",
    "Chithirai", "Swathi", "Visakam", "Anusham", "Kettai", "Moolam", "Pooradam",
    "Uthiradam", "Thiruvonam", "Avittam", "Sadayam", "Poorattathi", "Uthirattathi", "Revathi",
];

pub const YOGA_NAMES: [&str; 27] = [
    "Vishkambha", "Priti", "Ayushman", "Saubhagya", "Shobhana", "Atiganda", "Sukarma",
    "Dhriti", "Shula", "Ganda", "Vriddhi", "Dhruva", "Vyaghata", "Harshana", "Vajra",
    "Siddhi", "Vyatipata", "Variyan", "Parigha", "Shiva", "Siddha", "Sadhya", "Shubha",
    "Shukla", "Brahma", "Indra", "Vaidhriti",
];

pub const RASHI_NAMES: [&str; 12] = [
    "Mesha", "Vrishabha", "Mithuna", "Karka", "Simha", "Kanya", "Tula", "Vrishchika",
    "Dhanu", "Makara", "Kumbha", "Meena",
];

pub const WEEKDAY_NAMES: [&str; 7] = [
    "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
];

pub const PLANET_HORA_SEQUENCE: [&str; 7] = ["Sun", "Venus", "Mercury", "Moon", "Saturn", "Jupiter", "Mars"];

pub fn weekday_name(idx: usize) -> &'static str {
    WEEKDAY_NAMES[idx % 7]
}
