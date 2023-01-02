use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use clap::Parser;
use nt_hive2::Hive;
use simplelog::{Config, SimpleLogger};

#[derive(Parser)]
#[clap(name="cleanhive", author, version, about, long_about = None)]
struct Args {
    /// name of the file to dump
    pub(crate) hive_file: String,

    /// name of the first log file (extension LOG1) to merge. If not present
    /// we assume that there exists an equally named filed with extension `LOG1`
    /// in the same directory together with the hive file.
    pub(crate) log1_file: Option<String>,

    /// name of the first log file (extension LOG2) to merge. If not present
    /// we assume that there exists an equally named filed with extension `LOG2`
    /// in the same directory together with the hive file.
    pub(crate) log2_file: Option<String>,

    #[clap(flatten)]
    pub(crate) verbose: clap_verbosity_flag::Verbosity,

    /// name of the file to which the cleaned hive will be written.
    #[clap(short('O'), long("output"))]
    pub(crate) dst_hive: String,
}

pub fn main() -> Result<()> {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());

    let hive_file = PathBuf::from(&cli.hive_file);
    if !hive_file.exists() {
        bail!("missing hive file: {}", cli.hive_file);
    }
    let logfile1 = File::open(logfile_path(&hive_file, &cli.log1_file, "LOG1")?)?;
    let logfile2 = File::open(logfile_path(&hive_file, &cli.log2_file, "LOG2")?)?;

    let hive_file = File::open(hive_file)?;
    let mut hive = Hive::new(&hive_file, nt_hive2::HiveParseMode::NormalWithBaseBlock)?
        .with_transaction_log(logfile1.try_into()?)?
        .with_transaction_log(logfile2.try_into()?)?;

    let mut dst = File::create(cli.dst_hive)?;
    std::io::copy(&mut hive, &mut dst)?;
    Ok(())
}

fn logfile_path<T: AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr>>(
    hive_file: &Path,
    logfile: &Option<String>,
    extension: T,
) -> Result<PathBuf> {
    Ok(match logfile {
        Some(f) => {
            let p = PathBuf::from(f);
            if !p.exists() {
                bail!("missing log file: {}", f);
            }
            p
        }
        None => {
            let mut p = PathBuf::from(hive_file);
            let p_extension = match p.extension() {
                None => {
                    p.set_extension(extension);
                    return Ok(p);
                }
                Some(ext) => {
                    match ext.to_str() {
                        Some(x) => x.to_owned(),
                        None => {bail!("unable to convert filename")}
                    }
                }
            };

            p.set_extension(format!("{p_extension}.{extension}"));
            p
        }
    })
}
