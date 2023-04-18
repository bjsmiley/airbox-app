use std::{str::{FromStr}};
use num_enum::{TryFromPrimitive, IntoPrimitive};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use super::PeerId;



/// Represents public metadata about a peer. This is designed to hold information which is required among all applications using the P2P library.
/// This metadata is discovered through the discovery process or sent by the connecting device when establishing a new P2P connection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PeerMetadata {
	// pub name: String,
	// pub operating_system: Option<OperationSystem>,
	// pub version: Option<String>,
    pub name: String,
    pub typ: DeviceType,
    pub id: PeerId,
    pub addr: std::net::SocketAddr
    //pub ip: String,
    //pub port: u16
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum DeviceType {
    // XboxOne = 1,
    AppleiPhone = 6,
    AppleiPad = 7,
    AndroidDevice = 8,
    Windows10Desktop = 9,
    // Windows10Phone = 11,
    LinuxDevice = 12,
    // WindowsIoT = 13,
    // SurfaceHub = 14,
    WindowsLaptop = 15,
    // WindowsTablet = 16
}

impl From<DeviceType> for String {
    fn from(device_type: DeviceType) -> Self {
        match device_type {
            // DeviceType::XboxOne => "XboxOne".into(),
            DeviceType::AppleiPhone => "AppleiPhone".into(),
            DeviceType::AppleiPad => "AppleiPad".into(),
            DeviceType::AndroidDevice => "AndroidDevice".into(),
            DeviceType::Windows10Desktop => "Windows10Desktop".into(),
            // DeviceType::Windows10Phone => "Windows10Phone".into(),
            DeviceType::LinuxDevice => "LinuxDevice".into(),
            // DeviceType::WindowsIoT => "WindowsIoT".into(),
            // DeviceType::SurfaceHub => "SurfaceHub".into(),
            DeviceType::WindowsLaptop => "WindowsLaptop".into(),
            // DeviceType::WindowsTablet => "WindowsTablet".into()
        }
    }
}

impl FromStr for DeviceType {
    type Err = PeerMetadataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // "XboxOne" => Ok(DeviceType::XboxOne),
            "AppleiPhone" => Ok(DeviceType::AppleiPhone),
            "AppleiPad" => Ok(DeviceType::AppleiPad),
            "AndroidDevice" => Ok(DeviceType::AndroidDevice),
            "Windows10Desktop" => Ok(DeviceType::Windows10Desktop),
            // "Windows10Phone" => Ok(DeviceType::Windows10Phone),
            "LinuxDevice" => Ok(DeviceType::LinuxDevice),
            // "WindowsIoT" => Ok(DeviceType::WindowsIoT),
            // "SurfaceHub" => Ok(DeviceType::SurfaceHub),
            "WindowsLaptop" => Ok(DeviceType::WindowsLaptop),
            // "WindowsTablet" => Ok(DeviceType::WindowsTablet),
            _ => Err(PeerMetadataError::InvalidField("type".to_string()))
        }
    }
}

// impl PeerMetadata {
// 	pub fn from_hashmap(hashmap: &HashMap<String, String>) -> Result<Self, PeerMetadataError> {
// 		Ok(Self {
// 			device_id: hashmap.get("id").ok_or(PeerMetadataError::InvalidField("id".to_string()))?.to_string(),
//             device_name: hashmap.get("name").ok_or(PeerMetadataError::InvalidField("name".to_string()))?.to_string(),
//             device_type: hashmap.get("type").ok_or(PeerMetadataError::InvalidField("type".to_string()))?.parse()?
// 		})
// 	}

// 	pub fn to_hashmap(self) -> HashMap<String, String> {
// 		let mut hashmap = HashMap::new();
// 		hashmap.insert("name".to_string(), self.device_name);
//         hashmap.insert("id".to_string(), self.device_id);
//         hashmap.insert("type".to_string(), self.device_type.into());
// 		hashmap
// 	}
// }

/// Represents an error that can occur when creating a [PublicId] from a string.
#[derive(Error, Debug)]
pub enum PeerMetadataError {
	#[error("The hashmap is missing a valid field")]
	InvalidField(String),
}
