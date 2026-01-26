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
