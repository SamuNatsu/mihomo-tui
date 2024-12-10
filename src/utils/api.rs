#![allow(dead_code)]

use std::collections::BTreeMap;

use anyhow::Result;
use reqwest::{Client, Method, RequestBuilder};
use serde::Deserialize;
use serde_json::Value;

pub struct MihomoApi {
    api: String,
    secret: Option<String>,
}

impl MihomoApi {
    pub fn new<S>(api: S, secret: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            api: api.into(),
            secret: secret.map(|v| v.into()),
        }
    }

    fn create_request_builder(&self, method: Method, path: &str) -> RequestBuilder {
        let builder = Client::new().request(method, format!("{}{}", self.api, path));

        match &self.secret {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    // Logs
    // Traffic
    // Memory

    pub async fn get_version(&self) -> Result<String> {
        let body = self
            .create_request_builder(Method::GET, "/version")
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            version: String,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.version)
    }

    // Cache

    pub async fn clear_fake_ip_cache(&self) -> Result<()> {
        self.create_request_builder(Method::POST, "/cache/fakeip/flush")
            .send()
            .await?;

        Ok(())
    }

    // Running configuration

    pub async fn get_configs(&self) -> Result<Value> {
        let body = self
            .create_request_builder(Method::GET, "/configs")
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    pub async fn update_configs(&self, value: &Value) -> Result<()> {
        self.create_request_builder(Method::PUT, "/configs?force=true")
            .body(serde_json::to_string(value)?)
            .send()
            .await?;

        Ok(())
    }

    pub async fn patch_configs(&self, value: &Value) -> Result<()> {
        self.create_request_builder(Method::PATCH, "/configs")
            .body(serde_json::to_string(value)?)
            .send()
            .await?;

        Ok(())
    }

    pub async fn update_geo_database(&self) -> Result<()> {
        self.create_request_builder(Method::POST, "/configs/geo")
            .send()
            .await?;

        Ok(())
    }

    pub async fn restart(&self) -> Result<()> {
        self.create_request_builder(Method::POST, "/restart")
            .send()
            .await?;

        Ok(())
    }

    // Updates

    pub async fn upgrade_core(&self) -> Result<()> {
        self.create_request_builder(Method::POST, "/upgrade")
            .send()
            .await?;

        Ok(())
    }

    pub async fn upgrade_ui(&self) -> Result<()> {
        self.create_request_builder(Method::POST, "/upgrade/ui")
            .send()
            .await?;

        Ok(())
    }

    // Policy groups

    pub async fn get_groups(&self) -> Result<Vec<Value>> {
        let body = self
            .create_request_builder(Method::GET, "/group")
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            proxies: Vec<Value>,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.proxies)
    }

    pub async fn get_group_by_name(&self, name: &str) -> Result<Value> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!("/group/{}", urlencoding::encode(name)),
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    pub async fn clear_group_selection(&self, name: &str) -> Result<()> {
        self.create_request_builder(
            Method::DELETE,
            &format!("/group/{}", urlencoding::encode(name)),
        )
        .send()
        .await?;

        Ok(())
    }

    pub async fn test_group_delay(
        &self,
        name: &str,
        url: &str,
        timeout: u64,
    ) -> Result<BTreeMap<String, u64>> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!(
                    "/group/{}/delay?url={}&timeout={}",
                    urlencoding::encode(name),
                    urlencoding::encode(url),
                    timeout
                ),
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    // Proxies

    pub async fn get_proxies(&self) -> Result<BTreeMap<String, Value>> {
        let body = self
            .create_request_builder(Method::GET, "/proxies")
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            proxies: BTreeMap<String, Value>,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.proxies)
    }

    pub async fn get_proxy(&self, name: &str) -> Result<Value> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!("/proxies/{}", urlencoding::encode(name)),
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    pub async fn update_proxy(&self, name: &str, selection: &str) -> Result<()> {
        self.create_request_builder(
            Method::PUT,
            &format!("/proxies/{}", urlencoding::encode(name)),
        )
        .body(format!("{{\"name\":\"{}\"}}", selection))
        .send()
        .await?;

        Ok(())
    }

    pub async fn unselct_proxy(&self, name: &str) -> Result<()> {
        self.create_request_builder(
            Method::DELETE,
            &format!("/proxies/{}", urlencoding::encode(name)),
        )
        .send()
        .await?;

        Ok(())
    }

    pub async fn test_proxy_delay(&self, name: &str, url: &str, timeout: u64) -> Result<u64> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!(
                    "/proxies/{}/delay?url={}&timeout={}",
                    urlencoding::encode(name),
                    urlencoding::encode(url),
                    timeout
                ),
            )
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            delay: u64,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.delay)
    }

    // Proxy sets

    pub async fn get_proxy_sets(&self) -> Result<BTreeMap<String, Value>> {
        let body = self
            .create_request_builder(Method::GET, "/providers/proxies")
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            providers: BTreeMap<String, Value>,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.providers)
    }

    pub async fn get_proxy_set(&self, name: &str) -> Result<Value> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!("/providers/proxies/{}", urlencoding::encode(name)),
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    pub async fn update_proxy_set(&self, name: &str) -> Result<()> {
        self.create_request_builder(
            Method::PUT,
            &format!("/providers/proxies/{}", urlencoding::encode(name)),
        )
        .send()
        .await?;

        Ok(())
    }

    pub async fn health_check_provider_proxy(&self, name: &str) -> Result<Value> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!(
                    "/providers/proxies/{}/healthcheck",
                    urlencoding::encode(name)
                ),
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    // providers/proxies/providers_name/proxies_name/healthcheck

    // Rules

    pub async fn get_rules(&self) -> Result<Vec<Value>> {
        let body = self
            .create_request_builder(Method::GET, "/rules")
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            rules: Vec<Value>,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.rules)
    }

    // Rule sets

    pub async fn get_rule_sets(&self) -> Result<BTreeMap<String, Value>> {
        let body = self
            .create_request_builder(Method::GET, "/providers/rules")
            .send()
            .await?
            .text()
            .await?;

        #[derive(Deserialize)]
        struct Body {
            providers: BTreeMap<String, Value>,
        }

        let body = serde_json::from_str::<Body>(&body)?;
        Ok(body.providers)
    }

    pub async fn update_rule_set(&self, name: &str) -> Result<()> {
        self.create_request_builder(
            Method::PUT,
            &format!("/providers/rules/{}", urlencoding::encode(name)),
        )
        .send()
        .await?;

        Ok(())
    }

    // Connections

    pub async fn get_connections(&self) -> Result<Value> {
        let body = self
            .create_request_builder(Method::GET, "/connections")
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    pub async fn close_all_connections(&self) -> Result<()> {
        self.create_request_builder(Method::DELETE, "/connections")
            .send()
            .await?;

        Ok(())
    }

    pub async fn close_connection(&self, id: &str) -> Result<()> {
        self.create_request_builder(
            Method::DELETE,
            &format!("/connections/{}", urlencoding::encode(&id)),
        )
        .send()
        .await?;

        Ok(())
    }

    /// DNS

    pub async fn query_dns(&self, hostname: &str, record_type: &str) -> Result<Value> {
        let body = self
            .create_request_builder(
                Method::GET,
                &format!(
                    "/dns/query?name={}&type={}",
                    urlencoding::encode(hostname),
                    urlencoding::encode(record_type)
                ),
            )
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&body)?)
    }

    // Debug

    pub async fn debug_gc(&self) -> Result<()> {
        self.create_request_builder(Method::PUT, "/debug/gc")
            .send()
            .await?;

        Ok(())
    }

    // debug/pprof
}
