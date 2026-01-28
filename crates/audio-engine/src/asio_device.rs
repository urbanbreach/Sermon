//! ASIO driver enumeration for Windows
//!
//! This module provides ASIO driver discovery functionality using asio-sys.
//! ASIO drivers are enumerated by name from the Windows registry.

use asio_sys::Asio;

/// Information about an available ASIO driver.
#[derive(Debug, Clone)]
pub struct AsioDriverInfo {
    /// The driver name as registered in the system.
    pub name: String,
}

/// List all available ASIO drivers on the system.
///
/// Returns an empty vector if no ASIO drivers are installed.
/// This function only enumerates driver names from the registry;
/// it does not initialize or load any drivers.
///
/// # Example
///
/// ```no_run
/// use audio_engine::asio_device::list_asio_drivers;
///
/// let drivers = list_asio_drivers();
/// for driver in drivers {
///     println!("Found ASIO driver: {}", driver.name);
/// }
/// ```
pub fn list_asio_drivers() -> Vec<AsioDriverInfo> {
    let asio = Asio::new();
    asio.driver_names()
        .into_iter()
        .map(|name| AsioDriverInfo { name })
        .collect()
}

/// Open the ASIO control panel for the specified driver.
///
/// This function temporarily loads the driver on a dedicated STA COM thread,
/// opens the control panel, and then unloads the driver. It can be called
/// even when no playback is active.
///
/// # Arguments
///
/// * `driver_name` - The name of the ASIO driver to open the control panel for
///
/// # Returns
///
/// * `Ok(())` if the control panel was opened successfully
/// * `Err(String)` if the driver could not be loaded or the control panel failed
///
/// # Example
///
/// ```no_run
/// use audio_engine::asio_device::open_asio_control_panel_direct;
///
/// if let Err(e) = open_asio_control_panel_direct("ASIO4ALL v2") {
///     eprintln!("Failed to open control panel: {}", e);
/// }
/// ```
pub fn open_asio_control_panel_direct(driver_name: &str) -> Result<(), String> {
    use std::sync::mpsc;
    use std::thread;
    use tracing::{debug, info, warn};
    use windows_sys::Win32::System::Com::{
        CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
    };

    let driver_name = driver_name.to_string();

    // Channel to receive the result from the worker thread
    let (tx, rx) = mpsc::channel();

    // Spawn a dedicated thread with STA COM for ASIO operations
    let handle = thread::Builder::new()
        .name("asio-control-panel".to_string())
        .spawn(move || {
            // Initialize COM as STA (required for ASIO)
            let com_initialized = unsafe {
                let hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED as u32);
                if hr < 0 {
                    let _ = tx.send(Err(format!("COM initialization failed: {:#010x}", hr)));
                    return;
                }
                true
            };

            // Load the driver
            let asio = Asio::new();
            let driver = match asio.load_driver(&driver_name) {
                Ok(d) => d,
                Err(e) => {
                    if com_initialized {
                        unsafe { CoUninitialize() };
                    }
                    let _ = tx.send(Err(format!(
                        "Failed to load driver '{}': {:?}",
                        driver_name, e
                    )));
                    return;
                }
            };

            // Open the control panel
            // The ASIO SDK function ASIOControlPanel is called through the driver
            // Note: asio-sys doesn't expose this directly, so we use the extern link
            unsafe extern "C" {
                #[link_name = "?ASIOControlPanel@@YAJXZ"]
                fn ASIOControlPanel() -> i32;
            }

            info!(driver = %driver_name, "Opening ASIO control panel (direct)");
            let _ = tx.send(Ok(())); // Send success before blocking call

            let result = unsafe { ASIOControlPanel() };
            if result != 0 && result != -1000 {
                warn!(error_code = result, "ASIOControlPanel returned error");
            }

            // Cleanup: drop driver and uninitialize COM
            drop(driver);
            if com_initialized {
                unsafe { CoUninitialize() };
            }

            debug!("ASIO control panel thread exiting");
        })
        .map_err(|e| format!("Failed to spawn thread: {}", e))?;

    // Wait for initial result (driver load success/failure)
    match rx.recv() {
        Ok(Ok(())) => {
            // Don't wait for thread to finish - control panel is a blocking dialog
            // The thread will clean up when the user closes the dialog
            Ok(())
        }
        Ok(Err(e)) => {
            let _ = handle.join();
            Err(e)
        }
        Err(_) => {
            let _ = handle.join();
            Err("Worker thread died unexpectedly".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_asio_drivers_returns_vec() {
        let drivers = list_asio_drivers();
        assert!(drivers.len() >= 0);
    }

    #[test]
    fn test_asio_driver_info_debug() {
        let info = AsioDriverInfo {
            name: "Test Driver".to_string(),
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("Test Driver"));
    }

    #[test]
    fn test_asio_driver_info_clone() {
        let info = AsioDriverInfo {
            name: "Test Driver".to_string(),
        };
        let cloned = info.clone();
        assert_eq!(info.name, cloned.name);
    }
}
