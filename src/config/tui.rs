use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TuiConfig {
    pub config_version: u64,
    pub controller_api: String,
    pub mihomo_data_path: String,
    pub profiles: Vec<TuiProfileConfig>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            config_version: 1,
            controller_api: String::default(),
            mihomo_data_path: String::default(),
            profiles: Vec::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TuiProfileConfig {
    pub uuid: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub remote: Option<TuiProfileRemoteConfig>,
}

impl Default for TuiProfileConfig {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4().into(),
            name: String::default(),
            remote: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TuiProfileRemoteConfig {
    pub url: String,
    pub user_agent: String,
    pub update_interval: u64,
    pub use_system_proxy: bool,
    pub use_mihomo_proxy: bool,
    pub allow_invalid_certificates: bool,
}

impl Default for TuiProfileRemoteConfig {
    fn default() -> Self {
        Self {
            url: String::default(),
            user_agent: format!("mihomo-tui/v{} (clash-verge)", env!("CARGO_PKG_VERSION")),
            update_interval: 1440,
            use_system_proxy: false,
            use_mihomo_proxy: false,
            allow_invalid_certificates: false,
        }
    }
}

impl TuiProfileRemoteConfig {
    pub async fn fetch_raw(&self) -> Result<(Option<String>, String), String> {
        let r = reqwest::Client::new()
            .get(self.url.clone())
            .header("User-Agent", self.user_agent.clone())
            .send()
            .await
            .map_err(|err| format!("fail to fetch raw profile: {}", err))?;

        let sub_info = r.headers().get("subscription-userinfo").map(|v| v.to_str().into());
        let body = r
            .text()
            .await
            .map_err(|err| format!("fail to get response text: {}", err))?;

        Ok((sub_info, body))
    }
}
