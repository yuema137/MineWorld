//! The NOTICE written beside a record (step-19 §6.4, §18.5 SD-TW-d-12; `docs/DECISIONS.md` `DEP-31`):
//! attribution, NOAA's two requested citations, the licence, the statement that the data is modified
//! and not endorsed, and the provenance — the URL, the retrieval date, the exact command, the tool's
//! version, the input's byte count, the CSV's byte and line counts. No digest (QTWd-7) and no clock:
//! the same arguments write the same bytes.

/// Where GHCN-Daily's station files are served; `<station>.dly` is appended.
pub const SOURCE: &str = "https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/";

/// What the NOTICE records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance<'a> {
    /// The station.
    pub station: &'a str,
    /// The CSV's file name.
    pub csv_name: &'a str,
    /// The first and last year.
    pub years: (i32, i32),
    /// When the `.dly` file was retrieved, `YYYY-MM-DD` (UTC), as given.
    pub retrieved: &'a str,
    /// The command, as run.
    pub command: &'a str,
    /// The `.dly` file's size.
    pub dly_bytes: usize,
    /// The CSV's size.
    pub csv_bytes: usize,
    /// The CSV's lines, the header included.
    pub csv_lines: usize,
}

/// The NOTICE's text, LF-terminated.
pub fn notice(provenance: &Provenance<'_>) -> String {
    let Provenance {
        station,
        csv_name,
        years: (from, to),
        retrieved,
        command,
        dly_bytes,
        csv_bytes,
        csv_lines,
    } = provenance;
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "{csv_name}\n\
         \n\
         Daily weather of GHCN-Daily station {station}, {from}-01-01 to {to}-12-31.\n\
         \n\
         Source: NOAA National Centers for Environmental Information (NCEI), Global Historical\n\
         Climatology Network - Daily (GHCN-Daily), Version 3, station file\n\
         {SOURCE}{station}.dly\n\
         retrieved {retrieved} (UTC).\n\
         \n\
         Licence: NOAA data disseminated through NOAA's Open Data Dissemination program is\n\
         made available under the Creative Commons 1.0 Universal Public Domain Dedication\n\
         (CC0-1.0); it is a work of the US federal government. There are no restrictions on\n\
         the use of the data. NOAA requests attribution, which this file gives.\n\
         \n\
         MODIFIED DATA. This file is not original NOAA data. It was reshaped from the station\n\
         file above by MineWorld's tools/weather-fetch: one row per day, the elements TMAX,\n\
         TMIN, PRCP, AWND and WDF2 in GHCN-Daily's units, fog = WT01 or WT02 or WT21, thunder =\n\
         WT03, and every missing (-9999) or quality-flagged value left empty; its gaps were\n\
         reported, not filled. It is not endorsed by NOAA, and no affiliation with NOAA is\n\
         stated or implied.\n\
         \n\
         Citations requested by NOAA:\n\
         Menne, M.J., et al., 2012: An overview of the Global Historical Climatology\n\
         Network-Daily Database. Journal of Atmospheric and Oceanic Technology, 29, 897-910,\n\
         doi:10.1175/JTECH-D-11-00103.1.\n\
         Menne, M.J., et al., 2012: Global Historical Climatology Network - Daily (GHCN-Daily),\n\
         Version 3. NOAA National Climatic Data Center, doi:10.7289/V5D21VHZ.\n\
         \n\
         Provenance:\n\
         tool       mineworld-weather-fetch {version}\n\
         command    {command}\n\
         input      {station}.dly, {dly_bytes} bytes\n\
         output     {csv_name}, {csv_bytes} bytes, {csv_lines} lines (LF)\n\
         \n\
         See docs/DECISIONS.md DEP-31 and DEP-8.\n"
    )
}
