use crate::{
    Environment, Expression, Int, RuntimeError,
    expression::FileSize,
    libs::{BuiltinInfo, helper::check_exact_args_len, lazy_module::LazyModule},
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

pub fn from(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from", &args, 1, ctx)?;
    match args.into_iter().next().unwrap() {
        Expression::Integer(x) => Ok(Expression::FileSize(FileSize::from_bytes(x as u64))),
        Expression::Float(x) => Ok(Expression::FileSize(FileSize::from_bytes(x as u64))),
        Expression::FileSize(x) => Ok(Expression::FileSize(x)),
        Expression::String(x) => {
            if let Ok(n) = x.parse::<u64>() {
                Ok(Expression::FileSize(FileSize::from_bytes(n)))
            } else if let Some((num, unit)) = split_file_size(&x) {
                Ok(Expression::FileSize(FileSize::from_float(num, unit)))
            } else {
                Err(RuntimeError::common(
                    format!("could not convert {x:?} to a filesize").into(),
                    ctx.clone(),
                    0,
                ))
            }
        }
        otherwise => Err(RuntimeError::common(
            format!("could not convert {otherwise:?} to a filesize").into(),
            ctx.clone(),
            0,
        )),
    }
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

fn split_file_size(size_str: &str) -> Option<(f64, &'static str)> {
    let trimmed = size_str.trim();

    // 找到最后一个数字字符（含小数点）的位置，作为数字/单位分界
    let split_pos = trimmed.rfind(|c: char| c.is_ascii_digit() || c == '.')?;
    let (number_part, unit_part) = trimmed.split_at(split_pos + 1);

    let unit = unit_part.trim().to_uppercase();

    // 天然支持 K/KB/KiB、大小写混用等多种格式
    let canonical_unit: &'static str = match unit.as_str() {
        "" | "B" => "B",
        "K" | "KB" | "KIB" => "K",
        "M" | "MB" | "MIB" => "M",
        "G" | "GB" | "GIB" => "G",
        "T" | "TB" | "TIB" => "T",
        "P" | "PB" | "PIB" => "P",
        _ => return None,
    };

    let number: f64 = number_part.trim().parse().ok()?;

    Some((number, canonical_unit))
}

pub fn from_size_str(size_str: &str, ctx: &Expression) -> Result<FileSize, RuntimeError> {
    if let Some((num, unit)) = split_file_size(size_str) {
        Ok(FileSize::from_float(num, unit))
    } else {
        Err(RuntimeError::common(
            format!("could not convert {size_str:?} to a filesize").into(),
            ctx.clone(),
            0,
        ))
    }
}
