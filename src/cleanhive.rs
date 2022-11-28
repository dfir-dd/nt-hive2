use std::{path::{PathBuf, Path}, fmt::Display, fs::{File, self}};

use clap::Parser;
use anyhow::{Result, bail, anyhow};
use nt_hive2::{Hive, transcationlogs::transactionlogs::RecoverHive};
use simplelog::{SimpleLogger, Config};

#[derive(Parser)]
#[clap(name="cleanhive", author, version, about, long_about = None)]
struct Args {
    /// name of the file to dump
    pub (crate) hive_file: String,

    /// name of the first log file (extension LOG1) to merge. If not present
    /// we assume that there exists an equally named filed with extension `LOG1`
    /// in the same directory together with the hive file.
    pub (crate) log1_file: Option<String>,

    /// name of the first log file (extension LOG2) to merge. If not present
    /// we assume that there exists an equally named filed with extension `LOG2`
    /// in the same directory together with the hive file.
    pub (crate) log2_file: Option<String>,

    #[clap(flatten)]
    pub (crate) verbose: clap_verbosity_flag::Verbosity,

    /// name of the file to which the cleaned hive will be written. 
    #[clap(short('O'), long("output"))]
    pub (crate) dst_hive: String
}

pub fn main() -> Result<()> {
    let cli = Args::parse();
    let _ = SimpleLogger::init(cli.verbose.log_level_filter(), Config::default());
    
    let hive_file = PathBuf::from(&cli.hive_file);
    if ! hive_file.exists() { bail!("missing hive file: {}", cli.hive_file); }
    let logfile1 = logfile_path(&hive_file, &cli.log1_file, "LOG1")?;
    let logfile1 = logfile_path(&hive_file, &cli.log2_file, "LOG2")?;

    let hive_file = File::open(hive_file)?;
    let hive = Hive::new(&hive_file, nt_hive2::HiveParseMode::NormalWithBaseBlock)?;

    fs::write(
        cli.dst_hive,
        RecoverHive::default().recover_hive(hive, &logfile1.parent().unwrap().to_string_lossy()),
    )
    .map_err(|why| anyhow!(why))
}

fn logfile_path<T: AsRef<str> + Display + std::convert::AsRef<std::ffi::OsStr>>(hive_file: &Path, logfile: &Option<String>, extension: T) -> Result<PathBuf> {
    Ok(match logfile {
        Some(f) => {
            let p = PathBuf::from(f);
            if ! p.exists() { bail!("missing log file: {}", f); }
            p
        }
        None => {
            let mut p = PathBuf::from(hive_file);
            match p.extension() {
                Some(ext) => {
                    match ext.to_str() {
                        Some(x) => {p.set_extension(format!("{x}.{extension}"));}
                        None => {bail!("unable to convert filename");}
                    }
                }
                None => { p.set_extension(extension); }
            }
            p
        }
    })
}