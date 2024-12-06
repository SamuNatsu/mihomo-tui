use anyhow::Result;
use reqwest::{ClientBuilder, Proxy};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

use crate::utils::path::get_profiles_dir;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Profile {
    pub uuid: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub remote: Option<ProfileRemote>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub updated_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub expired_at: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub traffics: Option<ProfileTraffics>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4().into(),
            name: "New Profile".into(),
            remote: None,
            updated_at: None,
            expired_at: None,
            traffics: None,
        }
    }
}

impl Profile {
    pub async fn update(&mut self) -> Result<()> {
        if let Some(remote) = &self.remote {
            let mut builder =
                ClientBuilder::new().danger_accept_invalid_certs(remote.allow_invalid_certificates);
            if !remote.use_system_proxy && !remote.use_mihomo_proxy {
                builder = builder.no_proxy();
            } else if remote.use_mihomo_proxy {
                builder = builder.no_proxy().proxy(Proxy::all("")?);
            }

            let r = builder
                .build()?
                .get(remote.url.clone())
                .header("User-Agent", remote.user_agent.clone())
                .send()
                .await?;

            let header = r.headers().get("Subscription-Userinfo");
            if let Some(header) = header {
                let mut used: Option<u64> = None;
                let mut total: Option<u64> = None;
                let mut expired_at: Option<u64> = None;

                for item in header.to_str()?.split(';') {
                    let items: Vec<&str> = item.trim().split('=').collect();
                    if items.len() != 2 {
                        continue;
                    }

                    let value = match items.get(1).unwrap().parse::<u64>() {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    match items.get(0).unwrap() {
                        &"upload" | &"download" => {
                            if let Some(old) = used {
                                used = Some(old + value)
                            } else {
                                used = Some(value)
                            }
                        }
                        &"total" => total = Some(value),
                        &"expire" => expired_at = Some(value),
                        _ => (),
                    }
                }

                self.traffics = Some(ProfileTraffics { used, total });
                self.expired_at = expired_at;
            }

            let body = r.text().await?;
            let file_path = get_profiles_dir().join(format!("{}.yaml", self.uuid));
            let mut file = File::create(file_path).await?;
            file.write_all(body.as_bytes()).await?;
            file.flush().await?;
        }

        Ok(())
    }

    pub async fn read(&self) -> Result<Value> {
        let file_path = get_profiles_dir().join(format!("{}.yaml", self.uuid));
        let raw_text = fs::read_to_string(file_path).await?;

        Ok(serde_yaml::from_str::<Value>(&raw_text)?)
    }

    pub async fn update_script(&self, script: String) -> Result<()> {
        let file_path = get_profiles_dir().join(format!("{}.json", self.uuid));
        let mut file = File::create(file_path).await?;
        file.write_all(script.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfileRemote {
    pub url: String,
    pub user_agent: String,

    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub use_system_proxy: bool,

    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub use_mihomo_proxy: bool,

    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub allow_invalid_certificates: bool,
}

impl Default for ProfileRemote {
    fn default() -> Self {
        Self {
            url: "".into(),
            user_agent: format!("Mihomo-TUI/v{} (clash-verge)", env!("CARGO_PKG_VERSION")),
            use_system_proxy: false,
            use_mihomo_proxy: false,
            allow_invalid_certificates: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfileTraffics {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub used: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub total: Option<u64>,
}

fn is_false(b: &bool) -> bool {
    !b
}
