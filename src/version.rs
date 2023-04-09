use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::Destruct;
use std::ops::Range;
use std::str::FromStr;

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MinecraftVersion {
    Release(MinecraftVersionData),
    Snapshot(MinecraftVersionData),
}

#[derive(Debug)]
pub struct BadVersion;

impl Display for BadVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("bad version")
    }
}

impl Error for BadVersion {}

impl FromStr for MinecraftVersion {
    type Err = BadVersion;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" | "" => Ok(built::obtain(built::LATEST_RELEASE).unwrap()),
            "latest_snapshot" | "snapshot" => Ok(built::obtain(built::LATEST_SNAPSHOT).unwrap()),
            _ => built::obtain(s).ok_or(BadVersion),
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersionData {
    pub id: &'static str,
    pub url: &'static str,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
    pub sha1: &'static str,
    pub compliance_level: u8,
    pub server_url: &'static str,
}

impl PartialOrd for MinecraftVersionData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.release_time.partial_cmp(&other.release_time)
    }
}

impl Ord for MinecraftVersionData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.release_time.cmp(&other.release_time)
    }
}

#[derive(Clone, Copy)]
pub enum ServerFlavor {
    // Vanilla
    Vanilla,

    // Plugins
    /*
    /// If you use this, you are retarded
    Bukkit,
    Spigot,
     */
    Paper,
    /*
    Purpur,
    Pufferfish,

    // Mods
    Forge,
    Fabric,
     */
}

const PAPER_RANGE: Range<MinecraftVersionData> = (MinecraftVersion!("1.12"));

impl ServerFlavor {
    pub fn check_version(&self, version: &MinecraftVersion) -> bool {
        match self {
            Self::Vanilla => true,

            Self::Paper => {
                if let MinecraftVersion::Release(release) = version && PAPER_RANGE.contains(release) {
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn url(self, version: MinecraftVersion) -> Option<Url> {
        check(self.check_version(&version))?;

        match self {
            Self::Vanilla => match version {
                MinecraftVersion::Release(data) | MinecraftVersion::Snapshot(data) => {
                    Some(data.server_url.parse().unwrap())
                }
            },
            Self::Paper => None
        }
    }
}

use crate::util::check;
pub use built::version as MinecraftVersion;

mod built {
    #[derive(Clone, Copy)]
    pub(super) enum ConstMCVersion {
        Release(ConstMCVersionData),
        Snapshot(ConstMCVersionData),
    }

    #[derive(Clone, Copy)]
    pub(super) struct ConstMCVersionData {
        pub id: &'static str,
        pub url: &'static str,
        pub time: &'static str,
        pub release_time: &'static str,
        pub sha1: &'static str,
        pub compliance_level: u8,
        pub server_url: &'static str,
    }

    pub(super) fn obtain(str: &str) -> Option<super::MinecraftVersion> {
        VERSIONS.get(str).map(|a| *a).map(map_version)
    }

    pub(super) fn map_version(c: ConstMCVersion) -> super::MinecraftVersion {
        match c {
            ConstMCVersion::Release(data) => {
                super::MinecraftVersion::Release(map_version_data(data))
            }
            ConstMCVersion::Snapshot(data) => {
                super::MinecraftVersion::Snapshot(map_version_data(data))
            }
        }
    }

    fn map_version_data(
        ConstMCVersionData {
            id,
            url,
            time,
            release_time,
            sha1,
            compliance_level,
            server_url,
        }: ConstMCVersionData,
    ) -> super::MinecraftVersionData {
        super::MinecraftVersionData {
            id,
            url,
            sha1,
            compliance_level,
            time: time.parse().unwrap(),
            release_time: release_time.parse().unwrap(),
            server_url,
        }
    }

    include!(concat!(env!("OUT_DIR"), "/versions.rs"));
}
