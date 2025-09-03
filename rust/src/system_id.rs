use super::{Error};
use serde::Deserialize;
use std::process::Command;

#[cfg(target_os = "macos")]
#[allow(unused)]
pub fn get_system_id() -> Result<String, Error> {
    #[derive(Deserialize)]
    struct HardwareOverview {
        #[serde(rename = "platform_UUID")]
        platform_uuid: String,
    }

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(rename = "SPHardwareDataType")]
        sp_hardware_data_type: Vec<HardwareOverview>,
    }

    let output = Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .arg("-json")
        .output()
        .map_err(|e| Error::Default("Failed invoking system_profiler to lookup SPHardwareDataType".to_string()))?;
    let wrapper = serde_json::from_slice::<Wrapper>(&output.stdout).map_err(|e| Error::Serialization("Failed parsing system_profiler SPHardwareDataType lookup response".to_string()))?;
    match wrapper.sp_hardware_data_type.first() {
        None => Err(Error::Serialization("Failed finding a hardware datatype in system_profiler SPHardwareDataType lookup".to_string())),
        Some(hardware_overview) => Ok(hardware_overview.platform_uuid.clone()),
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(unused)]
pub fn get_system_id() -> Result<String, Error> {
    todo!("NIY")
}

#[cfg(test)]
mod tests {
    use crate::system_id::get_system_id;

    #[test]
    #[cfg(target_os = "macos")]
    fn can_get_system_id_on_macos() {
        let system_id = get_system_id().unwrap();
        assert_eq!(system_id.len(), 36);
    }
}
