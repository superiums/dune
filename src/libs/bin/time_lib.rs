use crate::{Environment, Expression};
use chrono::{
    DateTime, Datelike, Duration as ChronoDuration, FixedOffset, Local, NaiveDate, NaiveDateTime,
    NaiveTime, TimeZone, Timelike, Utc,
};
use common_macros::hash_map;
use std::{collections::BTreeMap, thread, time::Duration};

use crate::libs::helper::{check_args_len, check_exact_args_len, get_string_arg};
use crate::libs::lazy_module::LazyModule;
use crate::{RuntimeError, libs::BuiltinInfo, reg_info, reg_lazy};

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 基本时间获取
        sleep, display,
        // 时间分量获取（参数格式统一为 [datetime]，datetime 在前以支持管道）
        year, month, weekday, day, hour, minute, second, seconds,
        // 时间戳
        stamp, stamp_ms,
        // 格式化
        fmt,
        // 核心操作
        now, parse, add, diff, timezone, is_leap, to_string,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 基本时间获取
        sleep => "sleep for a given number of milliseconds [ms] or duration string (e.g. '1s', '2m')", "<duration>"
        display => "get preformatted datetime as map with time/date/datetime/etc.", "[datetime]"

        // 时间分量获取（参数格式统一为 [datetime]，datetime 在前以支持管道）
        year => "get year (current or from specified datetime)", "[datetime]"
        month => "get month (1-12)", "[datetime]"
        weekday => "get weekday (1-7, Monday=1)", "[datetime]"
        day => "get day of month (1-31)", "[datetime]"
        hour => "get hour (0-23)", "[datetime]"
        minute => "get minute (0-59)", "[datetime]"
        second => "get second (0-59)", "[datetime]"
        seconds => "get seconds since midnight", "[datetime]"

        // 时间戳
        stamp => "get Unix timestamp in seconds", "[datetime]"
        stamp_ms => "get Unix timestamp in milliseconds", "[datetime]"

        // 格式化
        fmt => "format datetime (current or specified) using chrono format string", "[datetime] <format_string>"

        // 核心操作
        now => "get current datetime as DateTime object or formatted string", "[format_string]"
        parse => "parse datetime string, optionally with a chrono format string", "<datetime_string> [format_string]"
        add => "add a signed duration string (e.g. '1d2h30m', '-1h') or integer seconds to a datetime (defaults to now)", "[datetime] <duration>"
        diff => "calculate difference between two datetimes in given unit", "<datetime1> [datetime2] <unit>"
        timezone => "convert datetime to a different timezone offset (in hours)", "[datetime] <offset_hours> [format_string]"
        is_leap => "check if a year is a leap year", "[year]"
        to_string => "convert DateTime to string", "<datetime> [format_string]"
    })
}

// Helper Functions
pub fn parse_datetime_arg(
    arg: Expression,
    ctx: &Expression,
) -> Result<NaiveDateTime, RuntimeError> {
    match arg {
        Expression::DateTime(dt) => Ok(dt),
        Expression::String(s) => {
            // Try parsing common shell date formats
            const DATETIME_FORMATS: &[&str] = &[
                "%Y-%m-%d %H:%M",
                "%Y-%m-%d %H:%M:%S",
                "%Y/%m/%d %H:%M:%S",
                "%d/%m/%Y %H:%M:%S",
                "%m/%d/%Y %H:%M:%S",
                "%Y-%m-%d %I:%M %p",
                "%Y/%m/%d %I:%M %p",
                "%d/%m/%Y %I:%M %p",
                "%m/%d/%Y %I:%M %p",
            ];
            for f in DATETIME_FORMATS {
                if let Ok(dt) = NaiveDateTime::parse_from_str(&s, f) {
                    return Ok(dt);
                }
            }
            if let Ok(dt) = NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                return Ok(dt.and_hms_opt(0, 0, 0).unwrap());
            }
            const TIME_FORMATS: &[&str] = &["%H:%M:%S", "%H:%M", "%I:%M %p"];
            for f in TIME_FORMATS {
                if let Ok(time) = NaiveTime::parse_from_str(&s, f) {
                    return Ok(NaiveDateTime::new(Local::now().date_naive(), time));
                }
            }
            // Try parsing common formats
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(dt.naive_local());
            }
            if let Ok(dt) = DateTime::parse_from_rfc2822(&s) {
                return Ok(dt.naive_local());
            }
            if let Ok(ts) = s.parse::<i64>() {
                return Ok(Utc.timestamp_opt(ts, 0).unwrap().naive_utc());
            }
            Err(RuntimeError::common(
                format!("Unrecognized datetime format: {s}").into(),
                ctx.clone(),
                0,
            ))
        }
        Expression::Integer(ts) => Ok(Utc.timestamp_opt(ts, 0).unwrap().naive_utc()),
        _ => Err(RuntimeError::common(
            "Expected DateTime, string or timestamp for datetime".into(),
            ctx.clone(),
            0,
        )),
    }
}

fn parse_duration_string(s: &str, ctx: &Expression) -> Result<Duration, RuntimeError> {
    let mut total_ms = 0u64;
    let mut num = 0u64;

    for c in s.chars() {
        if c.is_ascii_digit() {
            num = num
                .checked_mul(10)
                .and_then(|v| v.checked_add(c.to_digit(10).unwrap() as u64))
                .ok_or_else(|| {
                    RuntimeError::common(
                        format!("Duration number overflow in '{s}'").into(),
                        ctx.clone(),
                        0,
                    )
                })?;
        } else {
            let multiplier: u64 = match c.to_ascii_lowercase() {
                's' => 1_000,
                'm' => 60 * 1_000,
                'h' => 60 * 60 * 1_000,
                'd' => 24 * 60 * 60 * 1_000,
                _ => {
                    return Err(RuntimeError::common(
                        format!("Unknown duration unit: {c}").into(),
                        ctx.clone(),
                        0,
                    ));
                }
            };
            total_ms = total_ms.saturating_add(num.saturating_mul(multiplier));
            num = 0;
        }
    }

    // Handle case without unit, default to milliseconds
    if num > 0 {
        total_ms = total_ms.saturating_add(num);
    }

    Ok(Duration::from_millis(total_ms))
}

/// Supports signed duration strings like "1d2h30m10s" or "-1h30m" (for subtraction).
fn parse_signed_duration(s: &str, ctx: &Expression) -> Result<ChronoDuration, RuntimeError> {
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let std_duration = parse_duration_string(rest, ctx)?;
    let d = ChronoDuration::from_std(std_duration).map_err(|e| {
        RuntimeError::common(format!("Invalid duration: {e}").into(), ctx.clone(), 0)
    })?;
    Ok(if neg { -d } else { d })
}

// Basic Time Functions
fn sleep(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sleep", &args, 1, ctx)?;

    let duration = match &args[0] {
        Expression::Float(n) if *n > 0.0 => Duration::from_millis(*n as u64),
        Expression::Integer(n) if *n > 0 => Duration::from_millis(*n as u64),
        Expression::String(s) => parse_duration_string(s, ctx)?,
        otherwise => {
            return Err(RuntimeError::common(
                format!("expected positive number or duration string, got {otherwise}").into(),
                ctx.clone(),
                0,
            ));
        }
    };

    thread::sleep(duration);
    Ok(Expression::None)
}

fn display(
    _args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let now = Local::now();
    let naive = now.naive_local();
    Ok(Expression::from(hash_map! {
        String::from("time") => Expression::String(naive.time().format("%H:%M:%S").to_string()),
        String::from("timepm") => Expression::String(naive.format("%-I:%M %p").to_string()),
        String::from("date") => Expression::String(naive.date().format("%Y-%m-%d").to_string()),
        String::from("datetime") => Expression::String(naive.format("%Y-%m-%d %H:%M:%S").to_string()),
        String::from("rfc3339") => Expression::String(now.to_rfc3339()),
        String::from("rfc2822") => Expression::String(now.to_rfc2822()),
        String::from("week") => Expression::Integer(naive.iso_week().week() as i64),
        String::from("ordinal") => Expression::Integer(naive.ordinal() as i64),
        String::from("datetime_obj") => Expression::DateTime(naive),
    }))
}

// Time Component Functions
fn get_time_component<F>(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
    extractor: F,
) -> Result<Expression, RuntimeError>
where
    F: Fn(NaiveDateTime) -> i64,
{
    match args.len() {
        0 => Ok(Expression::Integer(extractor(Local::now().naive_local()))),
        1 => {
            let dt = parse_datetime_arg(args.into_iter().next().unwrap(), ctx)?;
            Ok(Expression::Integer(extractor(dt)))
        }
        _ => Err(RuntimeError::common(
            "Expected 0 or 1 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

fn year(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| dt.year() as i64)
}

fn month(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| dt.month() as i64)
}

fn weekday(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| {
        dt.weekday().num_days_from_monday() as i64 + 1
    })
}

fn day(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| dt.day() as i64)
}

fn hour(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| dt.hour() as i64)
}

fn minute(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| dt.minute() as i64)
}

fn second(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| dt.second() as i64)
}

fn seconds(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    get_time_component(args, env, ctx, |dt| {
        dt.time().num_seconds_from_midnight() as i64
    })
}

fn is_leap(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let year = match args.first() {
        Some(Expression::Integer(y)) => *y,
        Some(_) => {
            return Err(RuntimeError::common(
                "Year must be an integer".into(),
                ctx.clone(),
                0,
            ));
        }
        None => Local::now().year() as i64,
    };
    Ok(Expression::Boolean(
        NaiveDate::from_ymd_opt(year as i32, 1, 1)
            .map(|d| d.leap_year())
            .unwrap_or(false),
    ))
}

// Timestamp Functions
fn stamp_generic(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
    to_value: impl Fn(DateTime<Utc>) -> i64,
) -> Result<Expression, RuntimeError> {
    check_args_len("stamp", &args, 0..=1, ctx)?;
    let dt = if args.is_empty() {
        Utc::now()
    } else {
        parse_datetime_arg(args.into_iter().next().unwrap(), ctx)?.and_utc()
    };
    Ok(Expression::Integer(to_value(dt)))
}

fn stamp(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    stamp_generic(args, env, ctx, |dt| dt.timestamp())
}

fn stamp_ms(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    stamp_generic(args, env, ctx, |dt| dt.timestamp_millis())
}

// Formatting Functions
/// datetime 放在前面，以支持管道：`dt | time.fmt "%Y-%m-%d"`
fn fmt(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("fmt", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let a0 = it.next().unwrap();

    let (dt, format_str) = if let Some(a1) = it.next() {
        // 两个参数：第一个是 datetime，第二个是 format
        let dt = parse_datetime_arg(a0, ctx)?;
        let format = match a1 {
            Expression::String(s) => s,
            _ => {
                return Err(RuntimeError::common(
                    "fmt requires format string as second argument".into(),
                    ctx.clone(),
                    0,
                ));
            }
        };
        (dt, format)
    } else {
        // 单参数：只提供 format，datetime 默认当前时间
        let format = match a0 {
            Expression::String(s) => s,
            _ => {
                return Err(RuntimeError::common(
                    "fmt requires a format string".into(),
                    ctx.clone(),
                    0,
                ));
            }
        };
        (Local::now().naive_local(), format)
    };

    Ok(Expression::String(dt.format(&format_str).to_string()))
}

fn now(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    match args.len() {
        0 => Ok(Expression::DateTime(Local::now().naive_local())),
        1 => {
            let format = match &args[0] {
                Expression::String(s) => s,
                _ => {
                    return Err(RuntimeError::common(
                        "now requires optional format string".into(),
                        ctx.clone(),
                        0,
                    ));
                }
            };
            Ok(Expression::String(Local::now().format(format).to_string()))
        }
        _ => Err(RuntimeError::common(
            "now expects 0 or 1 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

/// datetime 放在前面，以支持管道：`dt | time.to_string "%Y-%m-%d"`
fn to_string(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("to_string", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();

    let dt = parse_datetime_arg(it.next().unwrap(), ctx)?;

    if let Some(a) = it.next() {
        match a {
            Expression::String(format) => Ok(Expression::String(dt.format(&format).to_string())),
            _ => Err(RuntimeError::common(
                "Expected format string".into(),
                ctx.clone(),
                0,
            )),
        }
    } else {
        // Default to RFC3339-like format
        Ok(Expression::String(
            dt.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
        ))
    }
}

// Parsing and Creation Functions
/// datetime 字符串放在前面，以支持管道：`"2024-01-01" | time.parse "%Y-%m-%d"`
pub fn parse(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("parse", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();

    let datetime_str = match it.next().unwrap() {
        Expression::String(s) => s,
        _ => {
            return Err(RuntimeError::common(
                "parse requires datetime string as first argument".into(),
                ctx.clone(),
                0,
            ));
        }
    };

    let format_str = if let Some(a) = it.next() {
        match a {
            Expression::String(s) => s,
            _ => {
                return Err(RuntimeError::common(
                    "parse requires format string as second argument".into(),
                    ctx.clone(),
                    0,
                ));
            }
        }
    } else {
        // Try to parse without format
        return Ok(Expression::DateTime(parse_datetime_arg(
            Expression::String(datetime_str),
            ctx,
        )?));
    };

    // Try parsing as NaiveDateTime
    if let Ok(dt) = NaiveDateTime::parse_from_str(&datetime_str, &format_str) {
        return Ok(Expression::DateTime(dt));
    }

    // Try parsing as NaiveDate
    if let Ok(date) = NaiveDate::parse_from_str(&datetime_str, &format_str) {
        return Ok(Expression::DateTime(date.and_hms_opt(0, 0, 0).unwrap()));
    }

    // Try parsing as NaiveTime
    if let Ok(time) = NaiveTime::parse_from_str(&datetime_str, &format_str) {
        let today = Local::now().date_naive();
        return Ok(Expression::DateTime(today.and_time(time)));
    }

    Err(RuntimeError::common(
        format!("Failed to parse datetime '{datetime_str}' with format '{format_str}'").into(),
        ctx.clone(),
        0,
    ))
}

// Time Arithmetic Functions
/// `time.add <duration>` — 以当前时间为基准
/// `time.add <datetime> <duration>` — datetime 在前，以支持管道：`dt | time.add "1h"`
fn add(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("add", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let a0 = it.next().unwrap();

    let (base_dt, duration_arg) = if let Some(a1) = it.next() {
        (parse_datetime_arg(a0, ctx)?, a1)
    } else {
        (Local::now().naive_local(), a0)
    };

    let duration = match duration_arg {
        Expression::String(dur) => parse_signed_duration(&dur, ctx)?,
        Expression::Integer(secs) => ChronoDuration::seconds(secs),
        e => {
            return Err(RuntimeError::common(
                format!(
                    "add expects a duration string (e.g. \"1d2h30m\", \"-1h\") or integer seconds, got {e}"
                )
                .into(),
                ctx.clone(),
                0,
            ));
        }
    };

    Ok(Expression::DateTime(base_dt + duration))
}

/// datetime1/datetime2 放在前面，unit 作为最后可选参数：
/// `time.diff <datetime1> <datetime2> [unit]`
fn diff(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("diff", &args, 2..=3, ctx)?;
    let mut it = args.into_iter();
    let dt1 = parse_datetime_arg(it.next().unwrap(), ctx)?;
    let second = it.next().unwrap();
    let (dt2, unit) = if let Some(third) = it.next() {
        (
            parse_datetime_arg(second, ctx)?,
            get_string_arg(third, ctx)?,
        )
    } else {
        (Local::now().naive_local(), get_string_arg(second, ctx)?)
    };

    let duration = dt2 - dt1;

    let value = match unit.as_str() {
        "ms" | "milliseconds" => duration.num_milliseconds(),
        "s" | "seconds" => duration.num_seconds(),
        "m" | "minutes" => duration.num_minutes(),
        "h" | "hours" => duration.num_hours(),
        "d" | "days" => duration.num_days(),
        "w" | "weeks" => duration.num_weeks(),
        _ => duration.num_seconds(),
    };

    Ok(Expression::Integer(value))
}

// Timezone Functions
/// `time.timezone <offset_hours> [format_string]` — 以当前时间为基准
/// `time.timezone <datetime> <offset_hours> [format_string]` — datetime 在前，以支持管道：
/// `dt | time.timezone 8`
fn timezone(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("timezone", &args, 1..=3, ctx)?;
    let mut it = args.into_iter();
    let a0 = it.next().unwrap();
    let a1 = it.next();

    let (base_dt, offset_expr) = match (a0, a1) {
        // 第二参数存在且是数字：第一参数是 datetime
        (dt_expr, Some(offset_expr @ (Expression::Integer(_) | Expression::Float(_)))) => {
            (parse_datetime_arg(dt_expr, ctx)?, offset_expr)
        }
        // 否则第一参数本身就是 offset，基准为当前时间；此时 a1（若存在）应为 format
        (offset_expr, format_opt) => {
            let base_dt = Local::now().naive_local();
            let offset_hours = match offset_expr {
                Expression::Integer(h) => h,
                Expression::Float(h) => h.round() as i64,
                _ => {
                    return Err(RuntimeError::common(
                        "timezone requires offset in hours".into(),
                        ctx.clone(),
                        0,
                    ));
                }
            };
            return apply_timezone(base_dt, offset_hours, format_opt, ctx);
        }
    };

    let offset_hours = match offset_expr {
        Expression::Integer(h) => h,
        Expression::Float(h) => h.round() as i64,
        _ => unreachable!(),
    };

    apply_timezone(base_dt, offset_hours, it.next(), ctx)
}

fn apply_timezone(
    base_dt: NaiveDateTime,
    offset_hours: i64,
    format_opt: Option<Expression>,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if !(-12..=14).contains(&offset_hours) {
        return Err(RuntimeError::common(
            "Timezone offset must be between -12 and +14 hours".into(),
            ctx.clone(),
            0,
        ));
    }

    let offset = FixedOffset::east_opt((offset_hours * 3600) as i32).ok_or(
        RuntimeError::common("Invalid timezone offset".into(), ctx.clone(), 0),
    )?;

    let dt = offset.from_utc_datetime(&base_dt).naive_local();

    match format_opt {
        Some(Expression::String(format)) => Ok(Expression::String(dt.format(&format).to_string())),
        Some(e) => Err(RuntimeError::common(
            format!("timezone expects a format string, got {e}").into(),
            ctx.clone(),
            0,
        )),
        None => Ok(Expression::DateTime(dt)),
    }
}
