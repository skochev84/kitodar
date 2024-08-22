use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Deserialize, Serialize, EnumString, Display, Eq, PartialEq, Clone, Copy, Debug)]
pub enum VmsVersion {
    XProtect2023R1,
    XProtect2023R2,
    XProtect2023R3,
    XProtect2024R1,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct User {
    pub user_name: String,
    pub vms_version: VmsVersion,
    pub server_type: String,
}

impl User {
    pub fn new(user_name: String, vms_version: VmsVersion) -> User {
        User {
            user_name,
            vms_version,
            server_type: "vms".to_owned(),
        }
    }

    pub fn get_global_id(&self) -> String {
        format!("{}", self.user_name.to_lowercase())
    }
}
