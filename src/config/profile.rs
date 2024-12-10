use std::{
    fs as std_fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Result};
use boa_engine::{js_string, property::Attribute, Source};
use reqwest::{ClientBuilder, Proxy};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

use crate::utils::{
    path::{get_data_dir, get_profiles_dir},
    script::create_context,
};

use super::tui::TuiConfig;

pub struct ProfileManager {}

impl ProfileManager {
    pub fn get_all() -> &'static Mutex<Vec<Profile>> {
        static INSTANCE: OnceLock<Mutex<Vec<Profile>>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let path = get_data_dir().join("profiles.yaml");

            if std_fs::exists(&path).unwrap() {
                let raw_str = std_fs::read_to_string(path).unwrap();
                Mutex::new(serde_yaml::from_str(&raw_str).unwrap())
            } else {
                Mutex::new(Vec::new())
            }
        })
    }

    pub async fn flush_all() -> Result<()> {
        let profiles = Self::get_all().lock().unwrap().clone();

        let path = get_data_dir().join("profiles.yaml");
        let mut file = File::create(path).await?;
        file.write_all(serde_yaml::to_string(&profiles)?.as_bytes())
            .await?;
        file.flush().await?;

        Ok(())
    }

    pub async fn update_all() -> Result<()> {
        for profile in Self::get_all().lock().unwrap().iter_mut() {
            let _ = profile.update().await;
        }

        Ok(())
    }

    pub async fn active_fallback_profile() -> Result<()> {
        // Create fallback profile
        let mut value = serde_yaml::from_str::<Value>("{}")?;
        Profile::apply_tui_config(&mut value).await?;

        // Rewrite mihomo config
        let path = PathBuf::from_str(
            &TuiConfig::global()
                .mihomo_data_dir
                .clone()
                .ok_or(anyhow!("mihomo data directory not set"))?,
        )?
        .join("config.yaml");
        let mut file = File::create(&path)
            .await
            .with_context(|| format!("could not create file `{}`", path.display()))?;
        file.write_all(serde_yaml::to_string(&value)?.as_bytes())
            .await
            .with_context(|| format!("could not write buffer for file `{}`", path.display()))?;
        file.flush()
            .await
            .with_context(|| format!("could not flush buffer for file `{}`", path.display()))?;

        // Reload mihomo
        TuiConfig::global()
            .get_mihomo_api()
            .restart()
            .await
            .with_context(|| "could not restart mihomo core")?;

        Ok(())
    }
}

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

    #[serde(skip)]
    pub updating: bool,
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
            updating: false,
        }
    }
}

impl Profile {
    pub async fn update(&mut self) -> Result<()> {
        if let Some(remote) = &self.remote {
            log::info!("updating remote profile \"{}\" ({})", self.name, self.uuid);
            self.updating = true;

            // Fetch remote profile
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
                .timeout(Duration::from_secs(5))
                .send()
                .await?;

            // Parse header
            let header = r.headers().get("Subscription-Userinfo");
            if let Some(header) = header {
                log::debug!("header detected: profile_uuid={}", self.uuid);

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

            // Parse body
            let body = r.text().await?;
            let file_path = get_profiles_dir().join(format!("{}.yaml", self.uuid));
            let mut file = File::create(file_path).await?;
            file.write_all(body.as_bytes()).await?;
            file.flush().await?;

            // Update timestamp
            self.updated_at = Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
        } else {
            log::info!(
                "Skip updating local profile \"{}\" ({})",
                self.name,
                self.uuid
            );
        }

        self.updating = false;
        Ok(())
    }

    pub async fn update_script(&self, script: String) -> Result<()> {
        let file_path = get_profiles_dir().join(format!("{}.js", self.uuid));
        let mut file = File::create(file_path).await?;
        file.write_all(script.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    pub async fn read_raw(&self) -> Result<String> {
        let path = get_profiles_dir().join(format!("{}.yaml", self.uuid));
        Ok(fs::read_to_string(path).await?)
    }

    pub async fn activate(&self) -> Result<()> {
        // Read profile
        let contents = self.read_raw().await?;
        let value = serde_yaml::from_str::<Value>(&contents)?;

        // Apply profile extend script
        let path = get_profiles_dir().join(format!("{}.js", self.uuid));
        let value = Self::apply_extend_scripts(&path, value).await?;

        // Apply global extend script
        let path = get_data_dir().join("global.js");
        let mut value = Self::apply_extend_scripts(&path, value).await?;

        // Apply TUI config
        Self::apply_tui_config(&mut value).await?;

        // Rewrite mihomo config
        let path = PathBuf::from_str(
            &TuiConfig::global()
                .mihomo_data_dir
                .clone()
                .ok_or(anyhow!("mihomo data directory not set"))?,
        )?
        .join("config.yaml");
        let mut file = File::create(path).await?;
        file.write_all(serde_yaml::to_string(&value)?.as_bytes())
            .await?;
        file.flush().await?;

        // Reload mihomo
        TuiConfig::global().get_mihomo_api().restart().await?;

        Ok(())
    }

    async fn apply_extend_scripts(path: &Path, value: Value) -> Result<Value> {
        // Check script existance
        if !fs::try_exists(&path).await? {
            return Ok(value);
        }

        // Read scripts
        let contents = fs::read_to_string(path).await?;
        let source = Source::from_bytes(contents.as_bytes());

        // Execute scripts
        let mut context = create_context()?;
        context
            .eval(source)
            .map_err(|err| err.into_erased(&mut context))?;

        // Inject configs to be processed
        let json = js_string!(serde_json::to_string(&value)?);
        context
            .register_global_property(js_string!("__CONFIG__"), json, Attribute::all())
            .map_err(|err| err.into_erased(&mut context))?;

        // Execute main function
        let source = Source::from_bytes("JSON.stringify(main(__CONFIG__))");
        let output = context
            .eval(source)
            .map_err(|err| err.into_erased(&mut context))?
            .to_string(&mut context)
            .map_err(|err| err.into_erased(&mut context))?
            .to_std_string()?;

        // Parse json
        Ok(serde_json::from_str(&output)?)
    }

    async fn apply_tui_config(value: &mut Value) -> Result<()> {
        let config = TuiConfig::global();

        // Set mode
        value
            .as_mapping_mut()
            .ok_or(anyhow!("config is not an object"))?
            .insert("mode".into(), config.mode.as_str().into());

        Ok(())
    }

    pub fn get_used_str(&self) -> Option<String> {
        if let Some(traffics) = &self.traffics {
            if traffics.total.is_none() || traffics.used.is_none() {
                None
            } else {
                Some(format!(
                    "{:.1}%",
                    traffics.used.unwrap() as f64 / traffics.total.unwrap() as f64 * 100.0
                ))
            }
        } else {
            None
        }
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
