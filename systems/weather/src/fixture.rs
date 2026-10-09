//! Tables the unit tests share: the provisional San Diego climate (step-19 §17.4, derived from NOAA's
//! 1991–2020 normals for USW00023188) and a constant table for the spell-length check.

use crate::rules::Rules;

/// The provisional San Diego table, as `configure/weather.yaml`'s `rules:` block.
pub const SAN_DIEGO: &str = "\
months:
  - { p_wet_after_dry: 147, p_wet_after_wet: 447, rain_tenth_mm: [8, 28, 54, 93, 178], tmax_dc: 191, tmin_dc: 102, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 80, thunder_permille: 40, overcast_morning_permille: 150, wind_dms: 25, wind_from_deg: 300 }
  - { p_wet_after_dry: 178, p_wet_after_wet: 478, rain_tenth_mm: [8, 28, 55, 95, 181], tmax_dc: 190, tmin_dc: 110, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 60, thunder_permille: 40, overcast_morning_permille: 150, wind_dms: 30, wind_from_deg: 290 }
  - { p_wet_after_dry: 140, p_wet_after_wet: 440, rain_tenth_mm: [6, 21, 41, 72, 138], tmax_dc: 194, tmin_dc: 125, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 50, thunder_permille: 40, overcast_morning_permille: 250, wind_dms: 34, wind_from_deg: 290 }
  - { p_wet_after_dry: 89, p_wet_after_wet: 389, rain_tenth_mm: [5, 16, 30, 52, 100], tmax_dc: 204, tmin_dc: 139, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 40, thunder_permille: 40, overcast_morning_permille: 400, wind_dms: 37, wind_from_deg: 290 }
  - { p_wet_after_dry: 50, p_wet_after_wet: 350, rain_tenth_mm: [3, 12, 22, 39, 74], tmax_dc: 208, tmin_dc: 156, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 40, thunder_permille: 40, overcast_morning_permille: 600, wind_dms: 37, wind_from_deg: 290 }
  - { p_wet_after_dry: 16, p_wet_after_wet: 316, rain_tenth_mm: [3, 6, 13, 22, 42], tmax_dc: 221, tmin_dc: 170, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 30, thunder_permille: 100, overcast_morning_permille: 650, wind_dms: 35, wind_from_deg: 290 }
  - { p_wet_after_dry: 16, p_wet_after_wet: 316, rain_tenth_mm: [3, 10, 20, 35, 67], tmax_dc: 241, tmin_dc: 189, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 30, thunder_permille: 300, overcast_morning_permille: 450, wind_dms: 34, wind_from_deg: 290 }
  - { p_wet_after_dry: 7, p_wet_after_wet: 307, rain_tenth_mm: [3, 3, 6, 10, 19], tmax_dc: 252, tmin_dc: 197, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 40, thunder_permille: 300, overcast_morning_permille: 350, wind_dms: 33, wind_from_deg: 290 }
  - { p_wet_after_dry: 21, p_wet_after_wet: 321, rain_tenth_mm: [4, 12, 23, 41, 78], tmax_dc: 251, tmin_dc: 190, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 60, thunder_permille: 250, overcast_morning_permille: 300, wind_dms: 31, wind_from_deg: 290 }
  - { p_wet_after_dry: 54, p_wet_after_wet: 354, rain_tenth_mm: [6, 19, 37, 64, 122], tmax_dc: 237, tmin_dc: 164, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 90, thunder_permille: 60, overcast_morning_permille: 250, wind_dms: 28, wind_from_deg: 290 }
  - { p_wet_after_dry: 86, p_wet_after_wet: 386, rain_tenth_mm: [6, 19, 38, 65, 125], tmax_dc: 215, tmin_dc: 127, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 90, thunder_permille: 40, overcast_morning_permille: 150, wind_dms: 25, wind_from_deg: 290 }
  - { p_wet_after_dry: 131, p_wet_after_wet: 431, rain_tenth_mm: [8, 26, 51, 88, 168], tmax_dc: 189, tmin_dc: 99, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, fog_permille: 90, thunder_permille: 40, overcast_morning_permille: 150, wind_dms: 24, wind_from_deg: 300 }
";

/// Decodes a `rules:` block as the loader would.
pub fn rules(text: &str) -> Rules {
    serde_saphyr::from_str(text).expect("the table decodes")
}

/// Every month the same: P(W|D) = `p_wd`, P(W|W) = `p_ww`, otherwise San Diego's January.
pub fn constant(p_wd: i64, p_ww: i64) -> Rules {
    let month = format!(
        "  - {{ p_wet_after_dry: {p_wd}, p_wet_after_wet: {p_ww}, rain_tenth_mm: [8, 28, 54, 93, 178], \
         tmax_dc: 191, tmin_dc: 102, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, \
         fog_permille: 80, thunder_permille: 40, overcast_morning_permille: 150, wind_dms: 25, \
         wind_from_deg: 300 }}\n"
    );
    rules(&format!("months:\n{}", month.repeat(12)))
}
