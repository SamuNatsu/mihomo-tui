mod config;

#[tokio::main]
async fn main() {
    let mut cfg = config::tui::TuiProfileRemoteConfig::default();
    cfg.url = "https://abc.xhonor.top:9066/v2b/hx/api/v1/client/subscribe?token=fe39b66ccf9f11d528cf19b04740c075".into();

    println!("{:?}", cfg.fetch_raw().await);
}
