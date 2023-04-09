#![feature(result_option_inspect)]

use chrono::{DateTime, Utc};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let versions = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/target/versions.json";
    println!("cargo:rerun-if-changed={versions}");
    write_versions_file(cache())
}

fn write_versions_file(versions: MinecraftVersions) {
    let versions_rs = std::env::var("OUT_DIR").unwrap() + "/versions.rs";
    let mut versions_rs = File::create(versions_rs).unwrap();

    let header = quote! {
        extern crate serde as __serde;
    };

    let constant = {
        let types = quote! {
            #[derive(Clone, Copy)]
            pub(super) enum ConstMCVersion {
                Release(ConstMCVersionData),
                Snapshot(ConstMCVersionData)
            }

            #[derive(Clone, Copy)]
            pub(super) struct ConstMCVersionData {
                pub id: &'static str,
                pub url: &'static str,
                pub time: &'static str,
                pub release_time: &'static str,
                pub sha1: &'static str,
                pub compliance_level: u8
            }
        };

        fn make_def_from_data(
            _MinecraftVersion {
                ref id,
                sha1,
                url,
                release_time,
                time,
                compliance_level,
            }: _MinecraftVersion,
            ver_type: TokenStream,
        ) -> TokenStream {
            let release_time = release_time.to_string();
            let time = time.to_string();
            quote!(#id => ConstMCVersion::#ver_type (ConstMCVersionData { id: #id, url: #url, sha1: #sha1, release_time: #release_time, time: #time, compliance_level: #compliance_level }))
        }

        let definitions = versions
            .versions
            .into_iter()
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

            #types

            static VERSIONS: __phf::Map<&'static str, ConstMCVersion> = __phf::phf_map! {
                #(#definitions,)*
            };

            pub(super) fn obtain(str: &str) -> Option<super::MinecraftVersion> {
                VERSIONS.get(str).map(|a| *a).map(map_version)
            }

            pub(super) fn map_version(c: ConstMCVersion) -> super::MinecraftVersion {
                match c {
                    ConstMCVersion::Release(data) => super::MinecraftVersion::Release(map_version_data(data)),
                    ConstMCVersion::Snapshot(data) => super::MinecraftVersion::Snapshot(map_version_data(data))
                }
            }

            fn map_version_data(ConstMCVersionData { id, url, time, release_time, sha1, compliance_level }: ConstMCVersionData) -> super::MinecraftVersionData {
                super::MinecraftVersionData {
                    id,
                    url,
                    sha1,
                    compliance_level,
                    time: time.parse().unwrap(),
                    release_time: release_time.parse().unwrap()
                }
            }

            #latest
        }
    };

    let tokens = quote! {
        #header

        mod constant {
            #constant
        }
    };
    versions_rs
        .write_all(tokens.to_string().as_bytes())
        .unwrap();
}

fn cache() -> MinecraftVersions {
    let versions = std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/target/versions.json";
    File::open(&versions)
        .map(|mut file| {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            println!("{}", &s);
            serde_json::from_str(&s).unwrap()
        })
        .unwrap_or_else(|_| download(versions))
}

fn download(versions_file: String) -> MinecraftVersions {
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(versions_file)
        .unwrap();
    let resp =
        reqwest::blocking::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .unwrap()
            .text()
            .unwrap();
    file.write_all(resp.as_bytes()).unwrap();
    serde_json::from_str(&resp)
        .inspect(|a| println!("{a:#?}"))
        .unwrap()
}

#[derive(Deserialize, Debug)]
struct MinecraftVersions {
    latest: Latest,
    versions: Vec<MinecraftVersion>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MinecraftVersion {
    Snapshot(_MinecraftVersion),
    Release(_MinecraftVersion),
    OldBeta(_MinecraftVersion),
    OldAlpha(_MinecraftVersion),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct _MinecraftVersion {
    id: String,
    url: String,
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    sha1: String,
    compliance_level: u8,
}

#[derive(Deserialize, Debug)]
struct Latest {
    release: String,
    snapshot: String,
}
