use anyhow::{anyhow, Result};
use boa_engine::{property::Attribute, vm::RuntimeLimits, Context};
use boa_runtime::Console;

pub fn create_context() -> Result<Context> {
    let mut runtime_limits = RuntimeLimits::default();
    

    let mut context = Context::default();
    context.set_runtime_limits(runtime_limits);

    let console = Console::init(&mut context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(context)
}
