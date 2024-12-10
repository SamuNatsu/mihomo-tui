use anyhow::{anyhow, Result};
use boa_engine::{property::Attribute, vm::RuntimeLimits, Context};
use boa_runtime::Console;

const RUNTIME_LOOP_LIMIT: u64 = 100_000;
const RUNTIME_RECURSION_LIMIT: usize = 100_000;
const RUNTIME_STACK_LIMIT: usize = 33_554_432; // 32MB

pub fn create_context() -> Result<Context> {
    let mut runtime_limits = RuntimeLimits::default();
    runtime_limits.set_loop_iteration_limit(RUNTIME_LOOP_LIMIT);
    runtime_limits.set_recursion_limit(RUNTIME_RECURSION_LIMIT);
    runtime_limits.set_stack_size_limit(RUNTIME_STACK_LIMIT);

    let mut context = Context::default();
    context.set_runtime_limits(runtime_limits);
    context.strict(true);

    let console = Console::init(&mut context);
    context
        .register_global_property(Console::NAME, console, Attribute::all())
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(context)
}
