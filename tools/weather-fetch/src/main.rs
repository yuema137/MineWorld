//! The `weather-fetch` command. See the library's documentation for the modes; this file only reads
//! arguments and files and prints.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use mineworld_weather_fetch::fit::{RecordBlock, fit};
use mineworld_weather_fetch::{date_argument, reshape_bytes, write};

/// Makes a World Pack's weather record from NOAA GHCN-Daily (docs/DECISIONS.md DEP-31).
#[derive(Parser)]
#[command(name = "weather-fetch", version)]
struct Command {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// A station's .dly file → NAME.csv and NOTICE in DIR, and a report on stdout.
    Reshape {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        station: String,
        #[arg(long)]
        from: i32,
        #[arg(long)]
        to: i32,
        /// When the .dly file was retrieved (UTC), YYYY-MM-DD; recorded in the NOTICE.
        #[arg(long, value_parser = date_argument)]
        retrieved: String,
        #[arg(long)]
        out_dir: PathBuf,
        /// The CSV's name without `.csv`; by default `<station, lower case>-<from>-<to>`.
        #[arg(long)]
        name: Option<String>,
    },
    /// A record CSV → the full configure/weather.yaml with fitted rules.
    Fit {
        #[arg(long)]
        input: PathBuf,
        /// The configuration whose seed and overcast_morning_permille are kept.
        #[arg(long)]
        base: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// With --data and --first-year: write `source: record` naming this record.
        #[arg(long, requires_all = ["data", "first_year"])]
        station: Option<String>,
        #[arg(long, requires = "station")]
        data: Option<String>,
        #[arg(long, requires = "station")]
        first_year: Option<i32>,
        #[arg(long, default_value = "rules", value_parser = ["rules", "none"])]
        fill: String,
    },
    /// Downloads a station's .dly file (only in a build with `--features fetch`).
    Fetch {
        #[arg(long)]
        station: String,
        #[arg(long)]
        out: PathBuf,
    },
}

fn main() -> ExitCode {
    match run(Command::parse().mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("weather-fetch: {error}");
            ExitCode::FAILURE
        }
    }
}

fn read(path: &PathBuf) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// The command line as given, after the program's own name, so the NOTICE does not depend on where
/// the binary lives.
fn command_line() -> String {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    format!("weather-fetch {}", arguments.join(" "))
}

fn run(mode: Mode) -> Result<(), String> {
    match mode {
        Mode::Reshape {
            input,
            station,
            from,
            to,
            retrieved,
            out_dir,
            name,
        } => {
            let name = name.unwrap_or_else(|| format!("{}-{from}-{to}", station.to_lowercase()));
            let written = reshape_bytes(
                &read(&input)?,
                &station,
                (from, to),
                &retrieved,
                &name,
                &command_line(),
            )?;
            write(&out_dir, &name, &written)?;
            print!("{}", written.report);
            println!(
                "wrote {} ({} bytes, {} lines) and NOTICE",
                out_dir.join(format!("{name}.csv")).display(),
                written.csv.len(),
                written.csv.lines().count()
            );
            Ok(())
        }
        Mode::Fit {
            input,
            base,
            out,
            station,
            data,
            first_year,
            fill,
        } => {
            let file = mineworld_weather::record::decode(&read(&input)?)
                .map_err(|error| format!("{}: {error}", input.display()))?;
            let base = String::from_utf8(read(&base)?)
                .map_err(|_| format!("{}: not text", base.display()))?;
            let record = match (station, data, first_year) {
                (Some(station), Some(data), Some(first_year)) => Some(RecordBlock {
                    station,
                    data,
                    first_year,
                    fill,
                }),
                _ => None,
            };
            let text = fit(&file, &base, record)?;
            std::fs::write(&out, &text).map_err(|error| format!("{}: {error}", out.display()))?;
            println!("wrote {} ({} bytes)", out.display(), text.len());
            Ok(())
        }
        Mode::Fetch { station, out } => fetch(&station, &out),
    }
}

#[cfg(feature = "fetch")]
fn fetch(station: &str, out: &std::path::Path) -> Result<(), String> {
    let fetched = mineworld_weather_fetch::fetch::fetch(station, out)?;
    println!("url        {}", fetched.url);
    println!("retrieved  {} (UTC)", fetched.retrieved);
    println!("bytes      {}", fetched.bytes);
    println!("wrote      {}", out.display());
    Ok(())
}

#[cfg(not(feature = "fetch"))]
fn fetch(station: &str, _: &std::path::Path) -> Result<(), String> {
    Err(format!(
        "this build has no `fetch` (it is off by default); rebuild with `--features fetch`, or download \
         {}{station}.dly by hand and run `reshape`",
        mineworld_weather_fetch::notice::SOURCE
    ))
}
