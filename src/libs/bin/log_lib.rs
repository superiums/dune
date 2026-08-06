use common_macros::b_tree_map;

use crate::{Environment, Expression};

use crate::libs::BuiltinInfo;
use crate::libs::helper::{check_exact_args_len, get_string_arg};
use crate::libs::lazy_module::LazyModule;
use crate::{Int, RuntimeError, RuntimeErrorKind, reg_info, reg_lazy};

use std::collections::BTreeMap;
use std::sync::{LazyLock, RwLock};

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 日志级别控制
        levels, enable, level , disable , is_enabled ,
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
        level => "get/set the log level", "[int]"
        enable => "enable all log, or level", "[int]"
        disable => "disable all log", ""
        is_enabled => "log level is enabled?", "<level>"

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
const NONE: Int = 0;
const ERROR: Int = 1;
const WARN: Int = 2;
const INFO: Int = 3;
const DEBUG: Int = 4;
const TRACE: Int = 5;

static LOG_LEVEL: LazyLock<RwLock<Int>> = LazyLock::new(|| RwLock::new(INFO));

// Helper Functions
// 检查日志级别是否启用
fn is_log_level_enabled(level: Int) -> bool {
    *LOG_LEVEL.read().unwrap() >= level
}
// 日志级别管理函数
fn enable(
    args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if let Some(Expression::Integer(i)) = args.get(0) {
        *LOG_LEVEL.write().unwrap() = *i;
    } else {
        *LOG_LEVEL.write().unwrap() = TRACE;
    }
    Ok(Expression::None)
}
fn levels(
    _args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    return Ok(Expression::from(b_tree_map! {
        String::from("none") => NONE,
         String::from("error") => ERROR,
         String::from("warn") => WARN,
         String::from("info") => INFO,
         String::from("debug") => DEBUG,
         String::from("trace") => TRACE
    }));
}
fn level(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    // 查询
    if args.is_empty() {
        return Ok(Expression::Integer(*LOG_LEVEL.read().unwrap()));
    }
    // 设置
    if let Expression::Integer(level) = &args[0] {
        *LOG_LEVEL.write().unwrap() = *level;
        Ok(Expression::None)
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "Integer".into(),
                sym: args[0].to_string(),
                found: args[0].type_name(),
            },
            ctx.clone(),
            0,
        ))
    }
}

fn disable(
    _args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    *LOG_LEVEL.write().unwrap() = NONE;
    Ok(Expression::None)
}

fn is_enabled(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_enabled", &args, 1, ctx)?;

    if let Expression::Integer(level) = &args[0] {
        Ok(Expression::Boolean(is_log_level_enabled(*level)))
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "Integer".into(),
                sym: args[0].to_string(),
                found: args[0].type_name(),
            },
            ctx.clone(),
            0,
        ))
    }
}
// 通用日志打印函数
fn log_message(
    level: Int,
    prefix: &str,
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if !is_log_level_enabled(level) {
        return Ok(Expression::None);
    }

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

    // 处理多行输出
    for (i, line) in output.lines().enumerate() {
        if i == 0 {
            println!("{prefix}{line}");
        } else {
            println!("{}{}", " ".repeat(prefix.len()), line);
        }
    }

    // 处理没有换行符的结尾
    if !output.ends_with('\n') && !output.is_empty() {
        println!();
    }

    Ok(Expression::None)
}
// 各日志级别专用函数
fn info(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(INFO, "\x1b[92m[INFO] \x1b[m", args, env, ctx)
}

fn warn(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(WARN, "\x1b[93m[WARN] \x1b[m", args, env, ctx)
}

fn debug(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(DEBUG, "\x1b[94m[DEBUG]\x1b[m ", args, env, ctx)
}

fn error(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(ERROR, "\x1b[91m[ERROR]\x1b[m ", args, env, ctx)
}

fn trace(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    log_message(TRACE, "\x1b[95m[TRACE]\x1b[m ", args, env, ctx)
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
