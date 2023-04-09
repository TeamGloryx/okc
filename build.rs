#![feature(result_option_inspect)]

use chrono::{DateTime, Utc};
use proc_macro2::TokenStream;
use quote::quote;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

fn main() {
    let versions = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/target/versions.json";
    println!("cargo:rerun-if-changed={versions}");
    let client = Client::new();
    write_versions_file(cache(&client))
}

fn write_versions_file(versions: MinecraftVersions) {
    let versions_rs = std::env::var("OUT_DIR").unwrap() + "/versions.rs";
    let mut versions_rs = File::create(versions_rs).unwrap();

    let header = quote! {
        extern crate serde as __serde;
    };

    let constant = {
        fn make_def_from_data(
            MinecraftVersionData {
                id,
                sha1,
                url,
                release_time,
                time,
                compliance_level,
                server_url,
            }: &MinecraftVersionData,
            ver_type: TokenStream,
        ) -> TokenStream {
            let release_time = release_time.to_string();
            let time = time.to_string();
            quote!(#id => ConstMCVersion::#ver_type (ConstMCVersionData { id: #id, url: #url, sha1: #sha1, release_time: #release_time, time: #time, compliance_level: #compliance_level, server_url: #server_url }))
        }

        let definitions = versions
            .versions
            .iter()
            .filter_map(|a| match a {
                MinecraftVersion::Release(data) => Some(make_def_from_data(data, quote!(Release))),
                MinecraftVersion::Snapshot(data) => {
                    Some(make_def_from_data(data, quote!(Snapshot)))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        let latest = {
            let Latest { release, snapshot } = versions.latest;
            quote! {
                const LATEST_RELEASE: &'static str = #release;
                const LATEST_SNAPSHOT: &'static str = #snapshot;
            }
        };

        quote! {
            extern crate phf as __phf;
            extern crate chrono as __chrono;

            pub(super) static VERSIONS: __phf::Map<&'static str, ConstMCVersion> = __phf::phf_map! {
                #(#definitions,)*
            };

            #latest
        }
    };

    let version_macro = {
        let definitions = versions.versions.iter().map(|a| match a {
            MinecraftVersion::Release(data)
            | MinecraftVersion::Snapshot(data) => data
        }).map(|MinecraftVersionData { id, .. }| quote!((#id) => { crate::version::MinecraftVersion::from_str(#id).unwrap() },)).collect::<Vec<_>>();
        quote! {
            macro version {
                #(#definitions)*
                () => { compile_error!("empty version") }
            }
        }
    };

    let tokens = quote! {
        #header
        #constant
        #version_macro
    };
    versions_rs
        .write_all(tokens.to_string().as_bytes())
        .unwrap();
}

fn cache(client: &Client) -> MinecraftVersions {
    let versions = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/target/versions.json";
    File::open(&versions)
        .map(|mut file| {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            println!("file = {}", &s);
            serde_json::from_str(&s).unwrap()
        })
        .unwrap_or_else(|_| download(versions, client))
}

fn download(versions_file: String, client: &Client) -> MinecraftVersions {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(versions_file)
        .unwrap();
    println!("resp blocked");
    let resp = client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .unwrap()
        .text()
        .unwrap();
    println!("resp unblocked; parsing");
    let st: _MinecraftVersions = serde_json::from_str(&resp).unwrap();
    println!("`st` parsed");

    let get_data = |_MCVersionData {
                        id,
                        url,
                        time,
                        release_time,
                        sha1,
                        compliance_level,
                    }: _MCVersionData|
     -> MinecraftVersionData {
        println!("fetching server url for {}", &id);
        let server_url = client
            .get(&url)
            .send()
            .unwrap()
            .json::<VersionData>()
            .unwrap()
            .downloads
            .server
            .url;
        println!("fetched server url for {}", &id);
        MinecraftVersionData {
            id,
            url,
            time,
            release_time,
            sha1,
            compliance_level,
            server_url,
        }
    };

    let smallest_release_time = DateTime::<Utc>::from_str("2012-03-29T22:00:00+00:00").unwrap();

    let real = MinecraftVersions {
        latest: st.latest,
        versions: st
            .versions
            .into_iter()
            .filter_map(|a| match a {
                _MCVersion::Release(release) if release.release_time >= smallest_release_time => Some(MinecraftVersion::Release(get_data(release))),
                _MCVersion::Snapshot(snapshot) => {
                    Some(MinecraftVersion::Snapshot(get_data(snapshot)))
                }
                _ => None,
            })
            .collect(),
    };
    let real_resp = serde_json::to_string(&real).unwrap();
    println!("real_resp = {}", &real_resp);
    file.write_all(real_resp.as_bytes()).unwrap();
    real
}

#[derive(Deserialize, Debug)]
struct _MinecraftVersions {
    latest: Latest,
    versions: Vec<_MCVersion>,
}

#[derive(Deserialize, Serialize)]
struct MinecraftVersions {
    latest: Latest,
    versions: Vec<MinecraftVersion>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum MinecraftVersion {
    Release(MinecraftVersionData),
    Snapshot(MinecraftVersionData),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum _MCVersion {
    Snapshot(_MCVersionData),
    Release(_MCVersionData),
    OldBeta(_MCVersionData),
    OldAlpha(_MCVersionData),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct _MCVersionData {
    id: String,
    url: String,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    sha1: String,
    compliance_level: u8,
}

#[derive(Deserialize, Serialize)]
struct MinecraftVersionData {
    id: String,
    url: String,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    sha1: String,
    compliance_level: u8,
    server_url: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Deserialize)]
struct VersionData {
    downloads: VersionDownloads,
}

#[derive(Deserialize)]
struct VersionDownloads {
    server: VDS,
}

#[derive(Deserialize)]
struct VDS {
    url: String,
}
