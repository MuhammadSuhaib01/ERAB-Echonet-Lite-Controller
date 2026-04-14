use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    AirConditioner,
    Lighting,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProperties {
    /// Power status: 0x30 = ON, 0x31 = OFF
    pub power_status: Option<bool>,
    /// Set temperature in celsius (0xB3 for AC)
    pub set_temp: Option<u8>,
    /// Measured room temperature in celsius (0xBB for AC)
    pub room_temp: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDevice {
    pub ip: String,
    pub name: String,
    pub eoj: String,
    pub device_type: DeviceType,
    pub properties: TargetProperties,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Targets {
    pub devices: Vec<TargetDevice>,
}

impl Targets {
    pub fn new() -> Self {
        Self { devices: Vec::new() }
    }

    pub fn add_or_update_device(&mut self, ip: String, eoj: String, device_type: DeviceType, properties: TargetProperties) {
        let name = match device_type {
            DeviceType::AirConditioner => format!("AC_{}", ip),
            DeviceType::Lighting => format!("Light_{}", ip),
            DeviceType::Other(ref t) => format!("{}_{}", t, ip),
        };

        if let Some(existing) = self.devices.iter_mut().find(|d| d.ip == ip && d.eoj == eoj) {
            existing.properties = properties;
        } else {
            self.devices.push(TargetDevice {
                ip,
                name,
                eoj,
                device_type,
                properties,
            });
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        let data = fs::read_to_string(path)?;
        let targets = serde_json::from_str(&data)?;
        Ok(targets)
    }
}

pub fn determine_device_type(eoj: &str) -> DeviceType {
    if eoj.starts_with("0130") {
        DeviceType::AirConditioner
    } else if eoj.starts_with("0290") || eoj.starts_with("02A3") {
        DeviceType::Lighting
    } else {
        DeviceType::Other("Unknown".to_string())
    }
}

pub fn parse_power_status(data: Option<&[u8]>) -> Option<bool> {
    data.and_then(|d| d.first().map(|&v| v == 0x30))
}

pub fn parse_temperature(data: Option<&[u8]>) -> Option<u8> {
    data.and_then(|d| d.first().copied())
}
