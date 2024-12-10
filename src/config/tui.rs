use std::{fs, sync::OnceLock};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::utils::{api::MihomoApi, path::get_data_dir};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TuiConfig {
    pub controller_api: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub controller_api_secret: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mihomo_data_dir: Option<String>,

    pub mode: TuiConfigMode,
}

impl TuiConfig {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<TuiConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let path = get_data_dir().join("config.yaml");

            if fs::exists(&path).unwrap() {
                serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
            } else {
                Self {
                    controller_api: "http://localhost:9090".into(),
                    controller_api_secret: None,
                    mihomo_data_dir: None,
                    mode: TuiConfigMode::Direct,
                }
            }
        })
    }

    pub async fn flush(&self) -> Result<()> {
        let file_path = get_data_dir().join("config.yaml");
        let mut file = File::create(file_path).await?;
        file.write_all(serde_yaml::to_string(&self)?.as_bytes())
            .await?;
        file.flush().await?;

        Ok(())
    }

    pub fn get_mihomo_api(&self) -> MihomoApi {
        MihomoApi::new(&self.controller_api, self.controller_api_secret.as_ref())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TuiConfigMode {
    Direct,
    Global,
    Rule,
}

impl TuiConfigMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Global => "global",
            Self::Rule => "rule",
        }
    }
}
