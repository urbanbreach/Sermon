use thiserror::Error;
use wasapi::{initialize_mta, Device, DeviceEnumerator, Direction};

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Device not found: {0}")]
    NotFound(String),

    #[error("Enumeration failed: {0}")]
    EnumerationFailed(String),
}

#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

pub fn list_devices() -> Result<Vec<AudioDeviceInfo>, DeviceError> {
    init_com()?;

    let enumerator = DeviceEnumerator::new().map_err(map_wasapi_error)?;
    let default_device = enumerator
        .get_default_device(&Direction::Render)
        .map_err(map_wasapi_error)?;
    let default_id = default_device.get_id().ok();

    let collection = enumerator
        .get_device_collection(&Direction::Render)
        .map_err(map_wasapi_error)?;

    let mut devices = Vec::new();
    for device in &collection {
        let device = device.map_err(map_wasapi_error)?;
        let id = device.get_id().map_err(map_wasapi_error)?;
        let name = device.get_friendlyname().map_err(map_wasapi_error)?;
        let is_default = default_id
            .as_deref()
            .is_some_and(|default_id| default_id == id.as_str());

        devices.push(AudioDeviceInfo {
            id,
            name,
            is_default,
        });
    }

    Ok(devices)
}

pub fn get_default_device() -> Result<Device, DeviceError> {
    init_com()?;

    let enumerator = DeviceEnumerator::new().map_err(map_wasapi_error)?;
    enumerator
        .get_default_device(&Direction::Render)
        .map_err(map_wasapi_error)
}

pub fn get_device_by_id(id: &str) -> Result<Device, DeviceError> {
    init_com()?;

    let enumerator = DeviceEnumerator::new().map_err(map_wasapi_error)?;
    enumerator
        .get_device(id)
        .map_err(|err| DeviceError::NotFound(err.to_string()))
}

fn init_com() -> Result<(), DeviceError> {
    initialize_mta()
        .ok()
        .map_err(|err| DeviceError::EnumerationFailed(err.to_string()))?;
    Ok(())
}

fn map_wasapi_error(err: wasapi::WasapiError) -> DeviceError {
    DeviceError::EnumerationFailed(err.to_string())
}
