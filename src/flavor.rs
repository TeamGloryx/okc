use chrono::{DateTime, Utc};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MinecraftVersion {
    Release(MinecraftVersionData),
    Snapshot(MinecraftVersionData)
}

impl FromStr for MinecraftVersion {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        constant::obtain(s).ok_or(())
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersionData {
    pub id: &'static str,
    pub url: &'static str,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
    pub sha1: &'static str,
    pub compliance_level: u8,
}

include!(concat!(env!("OUT_DIR"), "/versions.rs"));
