use super::{Client, Error, Node};
use crate::client::{DEFAULT_TIMEOUT_SECS, Format};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct System {
    client: Client,
    pub id: String,
}

impl System {
    pub(crate) fn new(client: Client, id: String) -> Self {
        Self { client, id }
    }

    pub fn node(&self, id: &str, name: &str, upstream: bool, token: Option<String>) -> Result<Arc<Node>, Error> {
        let upstream_arg = if upstream { "yes".to_string() } else { "no".to_string() };
        let token_arg = token.unwrap_or("$uuid".to_string());
        let args = vec![
            self.id.to_string(),
            id.to_string(),
            name.to_string(),
            upstream_arg,
            token_arg,
        ];
        let mut client = self.client.clone();
        let transaction = client.send(Format::Json, "nodes", &args)?;
        let _res = client.wait_for_response(transaction, Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;
        Ok(Arc::new(Node::new(client, self.clone(), id.to_string())))
    }
}

pub fn get_system_id() -> Result<String, Error> {
    use base64::prelude::*;
    let id = get_system_uuid()?;
    Ok(BASE64_STANDARD.encode(id.as_bytes()))
}

#[cfg(target_os = "macos")]
#[allow(unused)]
pub fn get_system_uuid() -> Result<Uuid, Error> {
    use serde::Deserialize;
    use std::process::Command;
    use std::str::FromStr;

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
    let wrapper = serde_json::from_slice::<Wrapper>(&output.stdout).map_err(|e| {
        Error::Serialization("Failed parsing system_profiler SPHardwareDataType lookup response".to_string())
    })?;
    match wrapper.sp_hardware_data_type.first() {
        None => Err(Error::Serialization(
            "Failed finding a hardware datatype in system_profiler SPHardwareDataType lookup".to_string(),
        )),
        Some(hardware_overview) => {
            let id = Uuid::from_str(&hardware_overview.platform_uuid)
                .map_err(|e| Error::Serialization("Failed parsing uuid".to_string()))?;
            Ok(id)
        }
    }
}

#[cfg(target_os = "linux")]
#[allow(unused)]
pub fn get_system_uuid() -> Result<Uuid, Error> {
    let model = get_model()?;
    let serial_number = get_serial_number()?;
    let model_plus_serial_number = format!("{model}:{serial_number}");
    let id = Uuid::new_v5(&Uuid::NAMESPACE_OID, model_plus_serial_number.as_bytes());
    Ok(id)
}

#[cfg(target_os = "linux")]
fn get_model() -> Result<String, Error> {
    const MODEL_FILENAME: &str = "/sys/firmware/devicetree/base/model";
    std::fs::read_to_string(MODEL_FILENAME)
        .map_err(|e| Error::Io(format!("Failed reading {MODEL_FILENAME} to obtain model ({e:?})")))
}

#[cfg(target_os = "linux")]
fn get_serial_number() -> Result<String, Error> {
    const SN_FILENAME: &str = "/sys/firmware/devicetree/base/serial-number";
    std::fs::read_to_string(SN_FILENAME)
        .map_err(|e| Error::Io(format!("Failed reading {SN_FILENAME} to obtain serial number ({e:?})")))
}

#[cfg(test)]
mod tests {
    use crate::system::get_system_id;

    #[test]
    #[cfg(target_os = "macos")]
    fn can_get_system_id_on_macos() {
        let _system_id = get_system_id().unwrap();
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn can_get_system_id_on_linux() {
        use std::path::Path;
        const MODEL_FILENAME: &str = "/sys/firmware/devicetree/base/model";
        const SN_FILENAME: &str = "/sys/firmware/devicetree/base/serial-number";
        if !Path::new(MODEL_FILENAME).exists() || !Path::new(SN_FILENAME).exists() {
            return; // Skip because device tree is hidden; we are probably running in the CI env
        }

        let _system_id = get_system_id().unwrap();
    }
}
