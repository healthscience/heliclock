use wasm_bindgen::prelude::*;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use astro::{coords, ecliptic, nutation, sun, time};

#[wasm_bindgen]
pub struct HeliCore;

#[wasm_bindgen]
impl HeliCore {
    /// 1. CALCULATE ORBITAL POSITION (0-360°)
    /// Replaces linear math with true elliptical ecliptic longitude.
    #[wasm_bindgen]
    pub fn get_orbital_degree(timestamp_ms: i64) -> f64 {
        let jd = timestamp_to_jd(timestamp_ms);

        // Get Sun's geocentric ecliptic position (radians)
        // In a Heliocentric view, Earth is exactly 180 degrees opposite the Sun
        let (sun_ecl, _dist) = sun::geocent_ecl_pos(jd);
        
        // Convert Sun longitude to degrees
        let earth_long = (sun_ecl.long.to_degrees()) % 360.0;
        
        earth_long
    }

    /// 2. CALCULATE ZENITH ANGLE (Degrees)
    /// Used for the "Light Potential" and Local Solar Noon
    #[wasm_bindgen]
    pub fn get_zenith_angle(lat: f64, lon: f64, timestamp_ms: i64) -> f64 {
        let jd = timestamp_to_jd(timestamp_ms);

        // Sun's geocentric ecliptic coordinates (radians)
        let (sun_ecl, _sun_earth_dist) = sun::geocent_ecl_pos(jd);

        // Account for Nutation and Obliquity
        let (nut_in_long, nut_in_oblq) = nutation::nutation(jd);
        let true_oblq = ecliptic::mn_oblq_laskar(jd) + nut_in_oblq;

        // Convert to equatorial coordinates
        let sun_long_true = sun_ecl.long + nut_in_long;
        let asc = coords::asc_frm_ecl(sun_long_true, sun_ecl.lat, true_oblq);
        let dec = coords::dec_frm_ecl(sun_long_true, sun_ecl.lat, true_oblq);

        // Local observer math
        let lat_rad = lat.to_radians();
        let lon_rad = lon.to_radians();
        let greenwich_sid = time::mn_sidr(jd);
        let hour_angle = coords::hr_angl_frm_observer_long(greenwich_sid, lon_rad, asc);

        // Calculate altitude above horizon
        let altitude = coords::alt_frm_eq(hour_angle, dec, lat_rad);
        
        // Zenith is the complement of altitude (90 - Alt)
        90.0 - altitude.to_degrees()
    }
}

/// INTERNAL HELPER: Convert JS timestamp (ms) to Julian Day (JD)
fn timestamp_to_jd(timestamp_ms: i64) -> f64 {
    let timestamp_secs = timestamp_ms / 1000;
    let timestamp_nanos = ((timestamp_ms % 1000) * 1_000_000) as u32;
    
    let dt = Utc.timestamp_opt(timestamp_secs, timestamp_nanos)
        .single()
        .expect("Invalid timestamp");

    let decimal_day = dt.day() as f64
        + (dt.hour() as f64) / 24.0
        + (dt.minute() as f64) / 1440.0
        + (dt.second() as f64) / 86400.0
        + (dt.nanosecond() as f64) / 86_400_000_000_000.0;

    let date = time::Date {
        year: dt.year() as i16,    // astro 2.0.0 uses i32
        month: dt.month() as u8,  // astro 2.0.0 uses i32
        decimal_day,
        cal_type: time::CalType::Gregorian,
    };
    
    time::julian_day(&date)
}

// --- New Heli-Clock Network Logic ---
#[wasm_bindgen]
pub struct HeliCoord {
    // The Location of Truth: 41.5° West
    // This is the "Prime Meridian" of the HOP Network.
    truth_longitude: f64,
}

#[wasm_bindgen]
impl HeliCoord {
    #[wasm_bindgen(constructor)]
    pub fn new() -> HeliCoord {
        HeliCoord {
            truth_longitude: -41.5,
        }
    }

    /// Instead of checking a "Date," the engine checks the 
    /// Earth's Ecliptic Longitude (L).
    /// When L = 0.0, the Network Great Orbit begins.
    pub fn is_network_equinox(&self, ecliptic_longitude: f64) -> bool {
        // In the model, 0.0 is the exact physical Equinox.
        (ecliptic_longitude - 0.0).abs() < 0.0001
    }

    /// Calculates the "Network Arc" based on the Sun's position 
    /// relative to the Truth Longitude (-41.5).
    pub fn get_network_arc(&self, current_solar_longitude: f64) -> f64 {
        // The Arc is the difference between where the Sun IS 
        // and where the TRUTH is.
        let raw_arc = (current_solar_longitude - self.truth_longitude + 180.0) % 360.0;
        raw_arc / 360.0
    }
}