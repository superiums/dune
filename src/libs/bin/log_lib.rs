use common_macros::b_tree_map;

use crate::{Environment, Expression};

use crate::libs::BuiltinInfo;
use crate::libs::helper::{check_exact_args_len, get_string_arg, get_string_ref};
use crate::libs::lazy_module::LazyModule;
use crate::{Int, RuntimeError, RuntimeErrorKind, reg_info, reg_lazy};

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 日志级别控制
        levels, level , is_level_enabled , file,
        // 日志记录函数
        info , warn , debug , error , trace ,
        // 原始输出
        echo ,
    })
}
pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({

        // 日志级别控制
        levels => "view all levels", ""
        level => "get/set the log level", "[int|str]"
        is_level_enabled => "log level is enabled?", "<level>"
        file => "get/set log file path", "[path]"

        // 日志记录函数
        info => "log info", "<msg>"
        warn => "log warning", "<msg>"
        debug => "log debug", "<msg>"
        error => "log error", "<msg>"
        trace => "log trace", "<msg>"

        // 原始输出
        echo => "print message without formatting", "<msg>"
    })
}

// 日志级别常量
const NONE: u8 = 0;
const ERROR: u8 = 1;
const WARN: u8 = 2;
const INFO: u8 = 3;
const DEBUG: u8 = 4;
const TRACE: u8 = 5;

static LOG_LEVEL: LazyLock<RwLock<u8>> = LazyLock::new(|| RwLock::new(INFO));

static LOG_FILE: LazyLock<RwLock<Option<PathBuf>>> = LazyLock::new(|| RwLock::new(None));

// Helper Functions
// 检查日志级别是否启用
fn is_log_level_enabled(level: u8) -> bool {
    *LOG_LEVEL.read().unwrap() >= level
}

fn levels(
    _args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    Ok(Expression::from(b_tree_map! {
        String::from("none") => NONE,
         String::from("error") => ERROR,
         String::from("warn") => WARN,
         String::from("info") => INFO,
         String::from("debug") => DEBUG,
         String::from("trace") => TRACE
    }))
}

fn get_level(level: &str) -> u8 {
    match level {
        "none" => NONE,
        "error" => ERROR,
        "warn" => WARN,
        "info" => INFO,
        "debug" => DEBUG,
        "trace" => TRACE,
        _ => NONE,
    }
}

fn level(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    // 查询
    if args.is_empty() {
        return Ok(Expression::Integer(*LOG_LEVEL.read().unwrap() as Int));
    }
    // 设置
    match &args[0] {
        Expression::Integer(level) => {
            *LOG_LEVEL.write().unwrap() = *level as u8;
        }
        Expression::String(level) | Expression::Symbol(level) => {
            *LOG_LEVEL.write().unwrap() = get_level(level.to_lowercase().as_str());
        }
        _ => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "String/Integer".into(),
                    sym: args[0].to_string(),
                    found: args[0].type_name(),
                },
                ctx.clone(),
                0,
            ));
        }
    }
    Ok(Expression::None)
}
fn file(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    // 查询
    if args.is_empty() {
        let p = LOG_FILE.read().unwrap().clone();
        return if let Some(path) = p {
            Ok(Expression::String(format!("{}", path.display())))
        } else {
            Ok(Expression::String("Not Setted".into()))
        };
    }
    // 设置
    let p = get_string_ref(&args[0], ctx)?;
    let path = crate::utils::abs(p, env);

    if !path.exists() {
        std::fs::File::create(&path)
            .map_err(|e| RuntimeError::from_io_error(e, "create file".into(), ctx.clone(), 0))?;
    }

    *LOG_FILE.write().unwrap() = Some(path);
    Ok(Expression::None)
}

fn is_level_enabled(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_level_enabled", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(level) => Ok(Expression::Boolean(is_log_level_enabled(*level as u8))),
        Expression::String(level) | Expression::Symbol(level) => Ok(Expression::Boolean(
            is_log_level_enabled(get_level(level.to_lowercase().as_str())),
        )),
        _ => {
            Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "String/Integer".into(),
                    sym: args[0].to_string(),
                    found: args[0].type_name(),
                },
                ctx.clone(),
                0,
            ))
        }
    }
}
// 通用日志打印函数
fn log_message(
    level: u8,
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if !is_log_level_enabled(level) {
        return Ok(Expression::None);
    }

    let output = args
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    let (clr, prefix) = match level {
        NONE => (90, "NONE"),
        ERROR => (91, "ERROR"),
        WARN => (93, "WARN"),
        INFO => (92, "INFO"),
        DEBUG => (94, "DEBUG"),
        TRACE => (95, "TRACE"),
        _ => unreachable!(),
    };

    // 处理多行输出
    for (i, line) in output.lines().enumerate() {
        if i == 0 {
            println!(
                "\x1b[37m{} \x1b[{clr}m[{prefix}] \x1b[m{line}",
                timestamp_ms()
            );
        } else {
            println!("{}{}", " ".repeat(prefix.len()), line);
        }
    }

    if let Some(path) = LOG_FILE.read().unwrap().clone() {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|e| RuntimeError::from_io_error(e, "open file".into(), ctx.clone(), 0))?;
        let final_str = format!("{} [{prefix}] {output}\n", timestamp_ms());
        file.write_all(final_str.as_bytes())
            .map_err(|e| RuntimeError::from_io_error(e, "write file".into(), ctx.clone(), 0))?;
    }

    // 处理没有换行符的结尾
    // if !output.ends_with('\n') && !output.is_empty() {
    //     println!();
    // }

    Ok(Expression::None)
}
// 各日志级别专用函数
fn info(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(INFO, args, env, ctx)
}

fn warn(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(WARN, args, env, ctx)
}

fn debug(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(DEBUG, args, env, ctx)
}

fn error(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(ERROR, args, env, ctx)
}

fn trace(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(TRACE, args, env, ctx)
}
// 简单回显函数
fn echo(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let mut output = String::new();
    let mut first_arg = true;

    for arg in args {
        let value = get_string_arg(arg, ctx)?;

        if !first_arg {
            output.push(' ');
        }

        output.push_str(&value);
        first_arg = false;
    }

    println!("{output}");
    Ok(Expression::None)
}

use std::time::SystemTime;

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
