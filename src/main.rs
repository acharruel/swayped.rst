use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use syslog_tracing::Syslog;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Default, Debug)]
struct Args {
    /// Swayped configuration file
    #[clap(short, long)]
    config_file: Option<String>,

    /// Use syslog flag
    #[clap(short = 'S', long)]
    syslog: bool,

    /// Dry run flag
    #[clap(short = 'd', long)]
    dry_run: bool,

    /// log level
    #[arg(long = "log")]
    #[arg(env = "SWAYPED_LOG")]
    #[arg(default_value = "info")]
    pub log_level: String,
}

pub fn setup_logging(log_level: &str, syslog: bool) -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::OFF.into())
        .from_env()?
        .add_directive(log_level.parse()?);

    if syslog {
        let identity = std::ffi::CStr::from_bytes_with_nul(b"swayped\0").unwrap();
        let (options, facility) = Default::default();
        let syslog = Syslog::new(identity, options, facility).unwrap();
        let layer = tracing_subscriber::fmt::layer().with_writer(syslog);
        let tracer = tracing_subscriber::registry().with(layer).with(filter);
        tracing::subscriber::set_global_default(tracer).context("Failed to set subscriber")?;
    } else {
        let layer = tracing_subscriber::fmt::layer().without_time();
        let tracer = tracing_subscriber::registry().with(layer).with(filter);
        tracing::subscriber::set_global_default(tracer).context("Failed to set subscriber")?;
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    setup_logging(&args.log_level, args.syslog)?;
    swayped::run(args.dry_run, args.config_file).await?;
    Ok(())
}
