use std::str::FromStr;
use clap::ValueEnum;

mod latest {
    pub const RELEASE: &str = "1.19.4"; //include_str!(concat!(env!("OUT_DIR"), "/latest/release.txt"));
    pub const SNAPSHOT: &str = "23w14a"; //include_str!(concat!(env!("OUT_DIR"), "/latest/snapshot.txt"));
}

pub enum Version {
    Release(String),
    Snapshot(String)
}

impl Version {
    fn latest() -> Self {
        Self::Release(latest::RELEASE.to_owned())
    }

    fn latest_snapshot() -> Self {
        Self::Snapshot(latest::SNAPSHOT.to_owned())
    }
}

fn require<E>(cond: bool, err: impl Fn() -> E) -> Result<(), E> {
    if cond {
        Ok(())
    } else {
        Err(err())
    }
}

impl FromStr for Version {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" | "" => Ok(Version::latest()),
            "snapshot" => Ok(Version::latest_snapshot()),
            real => {
                let mut real = real;
                real = real.strip_prefix("1.").ok_or_else(|| "the version does not start with 1.".to_owned())?;
                let (a, b) = real.split_once('.').ok_or_else(|| "no dot???".to_owned())?;
                require(a.parse::<u8>().is_ok(), || "not a number at pos 1.X.y".to_owned())?;
                require(b.parse::<u8>().is_ok(), || "not a number at pos 1.x.Y".to_owned())?;

                Ok(Self::Release(real.to_owned()))
            }
        }
    }
}