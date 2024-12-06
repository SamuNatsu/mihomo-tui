use anyhow::Result;
use utils::script::create_context;

mod config;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let mut profile = config::profile::Profile::default();
    let mut remote = config::profile::ProfileRemote::default();

    remote.url = "https://abc.xhonor.top:9066/v2b/hx/api/v1/client/subscribe?token=fe39b66ccf9f11d528cf19b04740c075".into();
    profile.remote = Some(remote);

    profile.update().await?;
    println!("{:?}", profile);

    let v = profile.read().await?;
    println!("{:?}", v);

    let j = serde_json::to_string_pretty(&v)?;
    println!("{}", j);

    let mut ctx = create_context()?;
    let code = r#"console.log(Object.getOwnPropertyNames(this))"#;

    match ctx.eval(boa_engine::Source::from_bytes(code.as_bytes())) {
        Ok(v) => println!("{:?}", v),
        Err(e) => println!("{:?}", e),
    }

    Ok(())
}
