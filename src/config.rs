use std::path::{Path, PathBuf};

use color_eyre::eyre::{Context, Result};
use matrix_sdk::ruma::UserId;
use once_cell::sync::Lazy;
use serde::Deserialize;

pub static CONFIG: Lazy<Config> =
    Lazy::new(|| load("./config.toml").expect("Failed to load config"));

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub synapse: Synapse,
    pub store: Store,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Synapse {
    /// Url of the homeserver
    pub url: url::Url,
    /// UserId of the bot user
    #[serde(deserialize_with = "user_id::deserialize")]
    pub user: Box<UserId>,
    /// Password of the bot user
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Store {
    /// on disk location where the state store will be located
    pub location: PathBuf,
    /// Optional password used to encrypt the state store
    pub password: Option<String>,
}

impl Store {
    pub fn session_file(&self) -> PathBuf {
        self.location.join("session.json")
    }
}

fn load(path: impl AsRef<Path>) -> Result<Config> {
    let buf = std::fs::read(path.as_ref()).wrap_err("Failed to open config file")?;

    let config = toml::from_slice(&buf)?;

    Ok(config)
}

mod user_id {
    use std::{fmt, marker::PhantomData};

    use matrix_sdk::ruma::UserId;
    use serde::{
        de::{self, Visitor},
        Deserializer,
    };

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Box<UserId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeserUserId(PhantomData<fn() -> UserId>);

        impl<'de> Visitor<'de> for DeserUserId {
            type Value = Box<UserId>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("matrix user id")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                UserId::parse(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_any(DeserUserId(PhantomData))
    }
}
