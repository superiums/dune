use crate::{
    Environment, Expression, Int, RuntimeError,
    expression::FileSize,
    libs::{BuiltinInfo, bin::into_lib, helper::check_exact_args_len, lazy_module::LazyModule},
    reg_info, reg_lazy,
};
use std::collections::BTreeMap;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        from, to_string, b,
        kb, mb, gb, tb,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        from => "to Filesize", "<size_str|byte_int>"
        to_string => "to human readable string", "<filesize>"
        b => "bytes", "<filesize>"
        kb => "kilobytes (integer, truncated)", "<filesize>"
        mb => "megabytes", "<filesize>"
        gb => "gigabytes", "<filesize>"
        tb => "terabytes", "<filesize>"
    })
}

fn from(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from", &args, 1, ctx)?;
    into_lib::filesize(args, env, ctx)
}

fn b(
    args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("b", &args, 1, _ctx)?;
    let s = get_fsize_arg(args.into_iter().next().unwrap(), env, _ctx)?;
    Ok(Expression::Integer(s.to_bytes() as Int))
}
fn kb(
    args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("kb", &args, 1, _ctx)?;
    let s = get_fsize_arg(args.into_iter().next().unwrap(), env, _ctx)?;

    Ok(Expression::Integer((s.to_bytes() >> 10) as Int))
}
fn mb(
    args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("mb", &args, 1, _ctx)?;
    let s = get_fsize_arg(args.into_iter().next().unwrap(), env, _ctx)?.to_bytes();
    let r = (s >> 20) as f64 + ((s >> 10) & 1023) as f64 * 0.0009765625;

    Ok(Expression::Float(r))
}
fn gb(
    args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("gb", &args, 1, _ctx)?;
    let s = get_fsize_arg(args.into_iter().next().unwrap(), env, _ctx)?.to_bytes();
    let r = (s >> 30) as f64 + ((s >> 20) & 1023) as f64 * 0.0009765625;

    Ok(Expression::Float(r))
}
fn tb(
    args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("tb", &args, 1, _ctx)?;
    let s = get_fsize_arg(args.into_iter().next().unwrap(), env, _ctx)?.to_bytes();
    let r = (s >> 40) as f64 + ((s >> 30) & 1023) as f64 * 0.0009765625;

    Ok(Expression::Float(r))
}
fn to_string(
    args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_string", &args, 1, _ctx)?;
    let s = get_fsize_arg(args.into_iter().next().unwrap(), env, _ctx)?;

    Ok(Expression::String(s.to_human_readable()))
}

fn get_fsize_arg(
    arg: Expression,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<FileSize, RuntimeError> {
    match arg {
        Expression::FileSize(s) => Ok(s),
        _ => Err(RuntimeError::common(
            "Filesize.bytes requires only Filesize as argument".into(),
            ctx.clone(),
            0,
        )),
    }
}
