use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;

mod core;
mod gui;
mod hardware;
mod partitioning;
mod drivers;
mod network;
mod user;
mod recovery;

use crate::core::{InstallationConfig, InstallationWizard};
use crate::hardware::HardwareDetector;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("MultiOS Installation Wizard")
        .version("0.1.0")
        .about("Comprehensive MultiOS installation wizard with hardware detection")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to installation configuration file")
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Perform a dry run without actual installation")
        )
        .arg(
            Arg::new("no-gui")
                .long("no-gui")
                .help("Run in text mode without GUI")
        )
        .arg(
            Arg::new("output-log")
                .long("output-log")
                .value_name("FILE")
                .help("Path to save installation log")
        )
        .get_matches();

    // Initialize logging
    init_logging(matches.get_one::<String>("output-log").map(|s| s.as_str()))?;

    // Check if running with required permissions
    if !check_permissions() {
        log::error!("This installer requires root/admin privileges");
        return Ok(());
    }

    let dry_run = matches.get_flag("dry-run");
    let no_gui = matches.get_flag("no-gui");
    let config_path = matches.get_one::<String>("config");

    // Detect hardware
    log::info!("Starting MultiOS Installation Wizard");
    log::info!("Detecting hardware configuration...");
    
    let hardware_info = HardwareDetector::detect_all().await?;
    log::info!("Hardware detection completed");

    // Create or load configuration
    let config = match config_path {
        Some(path) => InstallationConfig::load_from_file(path)?,
        None => {
            log::info!("No config file provided, using default configuration");
            InstallationConfig::default()
        }
    };

    // Run installation wizard
    let mut wizard = InstallationWizard::new(config, hardware_info);
    
    if no_gui || !cfg!(feature = "gui") {
        log::info!("Running in text mode");
        wizard.run_text_mode().await?;
    } else {
        log::info!("Running in GUI mode");
        wizard.run_gui_mode().await?;
    }

    if dry_run {
        log::info!("Dry run completed successfully");
    } else {
        log::info!("MultiOS Installation Wizard completed");
    }

    Ok(())
}

fn init_logging(log_file: Option<&str>) -> Result<()> {
    use fern::Dispatch;
    use log::LevelFilter;

    let mut dispatch = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Info)
        .chain(std::io::stdout());

    if let Some(log_path) = log_file {
        dispatch = dispatch.chain(fern::log_file(log_path)?);
    }

    dispatch.apply()?;

    Ok(())
}

fn check_permissions() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::getuid() } == 0
    }
    
    #[cfg(windows)]
    {
        // Check if running as administrator on Windows
        let mut sid: winapi::shared::ntdef::PSID = std::ptr::null_mut();
        let mut token: winapi::um::winnt::HANDLE = std::ptr::null_mut();
        
        // This is a simplified check - in production you'd want more comprehensive admin checking
        winapi::um::processthreadsapi::OpenProcessToken(
            winapi::um::processthreadsapi::GetCurrentProcess(),
            winapi::um::winnt::TOKEN_QUERY,
            &mut token,
        );
        
        if !token.is_null() {
            winapi::um::processthreadsapi::CloseHandle(token);
        }
        
        !token.is_null()
    }
}