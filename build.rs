use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs::File;
use std::io::Write;

fn main() {
    //if let Some(latest) = get_latest() {
    //    write_latest(latest);
    //}
}

fn cache() -> MinecraftVersions {
    let versions = std::env::var("CARGO_BUILD_TARGET_DIR").unwrap() + "/versions.txt";
    File::open(&versions).unwrap_or_else(|_| download(versions))
}

fn download(versions_file: String) -> File {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(&versions_file)
        .unwrap();
    file.write_all(
        reqwest::blocking::get(&versions_file)
            .unwrap()
            .text()
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    file
}

#[derive(Deserialize)]
struct MinecraftVersions {
    latest: Latest,
    versions: Vec<MinecraftVersion>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum MinecraftVersion {
    Snapshot(_MinecraftVersion),
    Release(_MinecraftVersion),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct _MinecraftVersion {
    id: String,
    url: String,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    sha1: String,
    compliance_level: u8,
}

#[derive(Deserialize)]
struct Latest {
    release: String,
    snapshot: String,
}
