//! `weather-fetch`: a developer tool that makes a World Pack's weather record from NOAA's data
//! (`docs/DECISIONS.md` `DEP-31`; design `.structured-coding/plans/mvp0/step-19-time-weather.md` §18).
//!
//! ```text
//! reshape  --input FILE.dly --station ID --from YYYY --to YYYY --retrieved YYYY-MM-DD --out-dir DIR
//!          a GHCN-Daily station file → <name>.csv (the weather pack's record format) and NOTICE, and a
//!          report of missing values, gaps and weather-type population; byte-reproducible
//! fit      --input FILE.csv --base configure/weather.yaml --out FILE.yaml
//!          the rules (WGEN-lite, integers) estimated from a record; byte-reproducible
//! fetch    --station ID --out FILE.dly          only with `--features fetch` (off by default)
//!          downloads NOAA's file verbatim and prints the URL, the UTC date and the byte count
//! ```
//!
//! Nothing a world runs depends on this crate: `run`, the server and the clients never open a network
//! connection for weather (`INV-TW-7`).

fn main() {}
