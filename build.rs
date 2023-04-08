use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use quote::{quote, ToTokens};

fn main() {

}

fn write_versions_file() {
    let versions_rs = std::env::var("OUT_DIR").unwrap() + "/versions.rs";
    let mut versions_rs = File::create(versions_rs).unwrap();

    let header = quote! {
        extern crate serde as __serde;
    };

    let serialize = quote!(#[derive(__serde::Serialize, __serde::Deserialize)]);

    let data_types = quote! {
        #serialize
        pub struct MinecraftVersions {
            pub latest: Latest,
            pub versions: Vec<MinecraftVersion>
        }

        #serialize
        #[serde(tag = "type", rename_all = "lowercase")]
        pub enum MinecraftVersion {
            Release(MinecraftVersionData),
            Snapshot(MinecraftVersionData)
        }

        #serialize
        #[serde(rename_all = "camelCase")]
        pub struct MinecraftVersionData {
            pub id: String,
            pub url: String,
            pub time: DateTime<Utc>,
            pub release_time: DateTime<Utc>,
            pub sha1: String,
            pub compliance_level: u8,
        }

        #serialize
        pub struct Latest {
            pub release: String,
            pub snapshot: String,
        }
    };

    let constant = {
        let types = quote! {
            
        };

        quote! {
            extern crate phf as __phf;

            const VERSIONS: __phf::Map<&'static str, MCVersion> = __phf::phf_map! {

            };
        }
    };

    let constant_accessors = {
        quote! {

        }
    };

    let tokens = quote! {
        #header

        #data_types

        mod constant {
            #constant
        }

        #constant_accessors
    };
    versions_rs.write_all(tokens.to_string().as_bytes()).unwrap();
}

fn cache() -> MinecraftVersions {
    let versions = std::env::var("CARGO_BUILD_TARGET_DIR").unwrap() + "/versions.json";
    File::open(&versions)
        .map(|mut file| {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            serde_json::from_str(&s).unwrap()
        })
        .unwrap_or_else(|_| download(versions))
}

fn download(versions_file: String) -> MinecraftVersions {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(&versions_file)
        .unwrap();
    let resp = reqwest::blocking::get(&versions_file)
        .unwrap()
        .text()
        .unwrap();
    file.write_all(resp.as_bytes()).unwrap();
    serde_json::from_str(&resp).unwrap()
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
