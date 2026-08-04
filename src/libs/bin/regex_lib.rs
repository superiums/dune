use crate::libs::BuiltinInfo;
use crate::libs::helper::check_exact_args_len;
use crate::libs::lazy_module::LazyModule;
use crate::{Environment, Expression, RuntimeError, reg_info, reg_lazy};
use regex_lite::Regex;
use std::collections::BTreeMap;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 匹配定位
        find, find_all,
        // 匹配验证
        is_match,
        // 捕获组操作
        capture, captures, named_captures,
        // 文本处理
        split, replace, replace_all,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 匹配定位
        find => "first match, returns {start,end,found}", "<pattern> <text>"
        find_all => "all matches, list of {start,end,found}", "<pattern> <text>"

        // 匹配验证
        is_match => "contains a match?", "<pattern> <text>"

        // 捕获组操作
        capture => "first match's groups [full,g1,g2,...]", "<pattern> <text>"
        captures => "all matches' groups [[full,g1,...],...]", "<pattern> <text>"
        named_captures => "named groups as map. e.g. g'(?<y>\\d+)'", "<pattern> <text>"

        // 文本处理
        split => "split by pattern", "<pattern> <text>"
        replace => "replace first match", "<text> <pattern> <replacement>"
        replace_all => "replace all matches", "<text> <pattern> <replacement>"
    })
}

// Helper Functions
fn get_r_args<'a>(
    args: &'a [Expression],
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<(Regex, &'a str), RuntimeError> {
    match (&args[0], &args[1]) {
        (Expression::Regex(r), Expression::String(t) | Expression::Symbol(t)) => {
            Ok((r.regex.clone(), t))
        }
        (Expression::String(t) | Expression::Symbol(t), Expression::Regex(r)) => {
            Ok((r.regex.clone(), t))
        }
        (
            Expression::String(r) | Expression::Symbol(r),
            Expression::String(t) | Expression::Symbol(t),
        ) => Ok((
            Regex::new(r).map_err(|e| {
                RuntimeError::common(format!("invalid regex pattern: {e}").into(), ctx.clone(), 0)
            })?,
            t,
        )),

        _ => Err(RuntimeError::common(
            "regex option requires Regex/pattern_string as first argument".into(),
            ctx.clone(),
            0,
        )),
    }
}
// 匹配验证函数
fn is_match(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("match", &args, 2, ctx)?;
    let (regex, text) = get_r_args(&args, env, ctx)?;

    Ok(Expression::Boolean(regex.is_match(text)))
}
// 查找定位函数
fn find(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("find", &args, 2, ctx)?;
    let (regex, text) = get_r_args(&args, env, ctx)?;

    regex.find(text).map_or(Ok(Expression::None), |m| {
        Ok(Expression::from(common_macros::b_tree_map! {
          String::from("start") => Expression::Integer(m.start() as i64),
          String::from("end") => Expression::Integer(m.end() as i64),
          String::from("found") => Expression::String(m.as_str().to_string()),
        }))
    })
}

fn find_all(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("find_all", &args, 2, ctx)?;
    let (regex, text) = get_r_args(&args, env, ctx)?;

    let matches: Vec<Expression> = regex
        .find_iter(text)
        .map(|m| {
            Expression::from(common_macros::b_tree_map! {
              String::from("start") => Expression::Integer(m.start() as i64),
              String::from("end") => Expression::Integer(m.end() as i64),
              String::from("found") => Expression::String(m.as_str().to_string()),
            })
        })
        .collect();
    Ok(Expression::from(matches))
}
// 捕获组操作函数
fn capture(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("capture", &args, 2, ctx)?;
    let (regex, text) = get_r_args(&args, env, ctx)?;

    regex.captures(text).map_or(Ok(Expression::None), |caps| {
        let groups: Vec<Expression> = (0..caps.len())
            .map(|i| {
                caps.get(i)
                    .map(|m| Expression::String(m.as_str().to_string()))
                    .unwrap_or(Expression::None)
            })
            .collect();
        Ok(Expression::from(groups))
    })
}

fn captures(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("captures", &args, 2, ctx)?;
    let (regex, text) = get_r_args(&args, env, ctx)?;

    let all_caps: Vec<Expression> = regex
        .captures_iter(text)
        .map(|caps| {
            let groups: Vec<Expression> = (0..caps.len())
                .map(|i| {
                    caps.get(i)
                        .map(|m| Expression::String(m.as_str().to_string()))
                        .unwrap_or(Expression::None)
                })
                .collect();
            Expression::from(groups)
        })
        .collect();
    Ok(Expression::from(all_caps))
}

fn named_captures(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("named_captures", &args, 2, ctx)?;
    let (re, text) = get_r_args(&args, env, ctx)?;

    if let Some(caps) = re.captures(text) {
        let mut found = std::collections::BTreeMap::new();
        for (i, name) in re.capture_names().enumerate().skip(1) {
            if let (Some(mat), Some(n)) = (caps.get(i), name) {
                found.insert(n.to_string(), Expression::String(mat.as_str().to_string()));
            }
        }
        return Ok(Expression::from(found));
    }
    Ok(Expression::None)
}
// 文本处理函数
fn split(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("split", &args, 2, ctx)?;
    let (regex, text) = get_r_args(&args, env, ctx)?;

    let parts: Vec<Expression> = regex
        .split(text)
        .map(|s| Expression::String(s.to_string()))
        .collect();
    Ok(Expression::from(parts))
}

fn replace(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("replace", &args, 3, ctx)?;
    let (regex, text, replacement) = get_replace_args(args, ctx)?;

    Ok(Expression::String(
        regex.replace(&text, replacement.as_str()).to_string(),
    ))
}

fn replace_all(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("replace_all", &args, 3, ctx)?;
    let (regex, text, replacement) = get_replace_args(args, ctx)?;

    Ok(Expression::String(
        regex.replace_all(&text, replacement.as_str()).to_string(),
    ))
}

fn get_replace_args(
    args: Vec<Expression>,
    ctx: &Expression,
) -> Result<(Regex, String, String), RuntimeError> {
    let mut it = args.into_iter();
    let first = it.next().unwrap();
    let second = it.next().unwrap();
    let last = it.next().unwrap();
    let (replacement, text, regex) = match first {
        Expression::Regex(regex) => match (second, last) {
            (
                Expression::String(rep) | Expression::Symbol(rep),
                Expression::String(t) | Expression::Symbol(t),
            ) => (rep, t, regex.regex.clone()),
            _ => {
                return Err(RuntimeError::common(
                    "regex option requires 2 text argument".into(),
                    ctx.clone(),
                    0,
                ));
            }
        },
        Expression::String(text) => match (second, last) {
            (Expression::String(s) | Expression::Symbol(s), Expression::Regex(r)) => {
                (s, text, r.regex.clone())
            }
            (Expression::Regex(r), Expression::String(s) | Expression::Symbol(s)) => {
                (s, text, r.regex.clone())
            }
            _ => {
                return Err(RuntimeError::common(
                    "regex option requires a regex argument".into(),
                    ctx.clone(),
                    0,
                ));
            }
        },

        _ => {
            return Err(RuntimeError::common(
                "regex option requires text as first argument".into(),
                ctx.clone(),
                0,
            ));
        }
    };

    Ok((regex, text, replacement))
}
