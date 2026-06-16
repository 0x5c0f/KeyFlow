//! Windows service implementation for KeyFlow.
//!
//! Uses the `windows-service` crate to properly handle the Windows Service
//! Control Manager (SCM) protocol. This fixes error 1053 (service did not
//! respond to start request in a timely fashion).

use crate::config::Config;
use crate::daemon;
use anyhow::Result;
use std::ffi::OsString;
use std::sync::mpsc;
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "KeyFlow";
const SERVICE_DISPLAY_NAME: &str = "KeyFlow - Non-paste password input assistant";
const SERVICE_DESCRIPTION: &str = "Simulates keystrokes to bypass paste-disabled password fields";

// Generate the FFI entry point
windows_service::define_windows_service!(ffi_service_main, service_main);

/// Entry point for Windows service mode.
pub fn run_service() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

/// Service main function called by the SCM.
fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service_inner() {
        log::error!("Service failed: {e}");
    }
}

/// Inner service implementation.
fn run_service_inner() -> Result<()> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, handler)?;

    // Report that the service is starting
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::from_secs(5),
        process_id: None,
    })?;

    // Load configuration
    let config_path = Config::default_path();
    let config = Config::load(&config_path).unwrap_or_else(|e| {
        log::warn!("Failed to load config: {e}, using defaults");
        Config {
            settings: Default::default(),
            bindings: vec![],
        }
    });

    // Report that the service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    })?;

    // Run daemon in a thread so we can listen for shutdown signal
    let daemon_handle = std::thread::spawn(move || {
        daemon::run(config)
    });

    // Wait for shutdown signal
    let _ = shutdown_rx.recv();
    log::info!("Service stop signal received");

    // Report that the service is stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

/// Install the Windows service.
pub fn install_service() -> Result<()> {
    use std::process::Command;

    let exe_path = std::env::current_exe()?;
    let bin_path = format!("\"{}\" service run", exe_path.display());

    let output = Command::new("sc.exe")
        .args([
            "create",
            SERVICE_NAME,
            "binPath=",
            &bin_path,
            "start=",
            "auto",
            "DisplayName=",
            SERVICE_DISPLAY_NAME,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create service: {stderr}");
    }

    // Set service description
    let _ = Command::new("sc.exe")
        .args(["description", SERVICE_NAME, SERVICE_DESCRIPTION])
        .output();

    println!("Service '{SERVICE_NAME}' installed successfully.");
    println!("  Start with: sc start {SERVICE_NAME}");
    println!("  Or: net start {SERVICE_NAME}");

    Ok(())
}

/// Uninstall the Windows service.
pub fn uninstall_service() -> Result<()> {
    use std::process::Command;

    // Stop the service first
    let _ = Command::new("sc.exe").args(["stop", SERVICE_NAME]).output();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let output = Command::new("sc.exe")
        .args(["delete", SERVICE_NAME])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to delete service: {stderr}");
    }

    println!("Service '{SERVICE_NAME}' removed successfully.");
    Ok(())
}
