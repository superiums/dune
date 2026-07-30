use regex_lite::Regex;

use crate::{
    Environment, Expression, Int, RuntimeError, RuntimeErrorKind,
    libs::{
        BuiltinInfo,
        bin::{
            colors::{COLOR_MAP, true_color_by_hex},
            top,
        },
        helper::{
            check_args_len, check_exact_args_len, get_integer_arg, get_integer_ref, get_string_arg,
            get_string_ref,
        },
        lazy_module::LazyModule,
    },
    reg_info, reg_lazy,
    utils::unescape_str,
};
use std::{collections::BTreeMap, sync::OnceLock};

use crate::libs::bin::into_lib::{
    filesize as to_filesize, float as to_float, int as to_int, strip as strip_ansi,
    table as to_table, time as to_time,
};
static QUOTED_RE: OnceLock<Regex> = OnceLock::new();

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // pprint,
        // 转换
        to_int, to_float, to_filesize, to_time, to_table,
        to_safe,
        // 基础检查
        is_ascii, is_ascii_control, is_ascii_punctuation, is_ascii_digit, is_ascii_hexdigit,
        is_empty, is_whitespace, is_alpha, is_alphanumeric, is_numeric, is_lower, is_upper, is_title, len,
        // 子串检查
        starts_with, ends_with, contains, position, get,
        // 分割操作
        split, split_at, chars, words, words_quoted, lines, paragraphs, concat,
        // 修改操作
        insert, repeat, replace, slice, strip_prefix, strip_suffix, trim, trim_start, trim_end, lower, upper, title,
        rev,
        // 高级操作
        max_len, grep,
        strip_ansi,
        escape, unescape,
        // 格式化
        pad_start, pad_end, center, wrap,
        // 样式
        href, bold, dim, italic, underline, blink, invert, strike,
        // 标准颜色
        black, red, green, yellow, blue, magenta, cyan, white,
        // 高级颜色
        clr, clr_bg, color, color_bg, colors,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 转换
        to_int => "to int. radix ok(0x/0o/0b), _ as sep. e.g. 0xff_80", "<value>"
        to_float => "to float. % as /100, _ as sep. e.g. 12.5%", "<value>"
        to_filesize => "to filesize. e.g. 1.5GB, 500K", "<size_str>"
        to_time => "to datetime", "<str> [fmt]"
        to_table => "parse cmd output to table", "<output> [regex] [headers...]"
        to_safe => "wrap str, never eval", "<str>"

        // 基础检查
        is_empty => "is empty?", "<string>"
        is_whitespace => "is whitespace?", "<string>"
        is_ascii => "is ascii?", "<string>"
        is_ascii_control => "is ascii control char?", "<string>"
        is_ascii_punctuation => "is ascii punctuation?", "<string>"
        is_ascii_digit => "is ascii digit?", "<string>"
        is_ascii_hexdigit => "is ascii hexdigit?", "<string>"
        is_alpha => "is alphabetic?", "<string>"
        is_alphanumeric => "is alphanumeric?", "<string>"
        is_numeric => "is numeric?", "<string>"
        is_lower => "is lowercase?", "<string>"
        is_upper => "is uppercase?", "<string>"
        is_title => "is title case?", "<string>"
        len => "char count", "<string>"

        // 子串检查
        starts_with => "starts with?", "<string> <substring>"
        ends_with => "ends with?", "<string> <substring>"
        contains => "contains?", "<string> <substring>"
        position => "index of substring, or None. search from [start]", "<string> <substring> [start]"
        get => "char at index. negative counts from end", "<string> <index>"

        // 分割操作
        split => "split by delimiter/whitespace", "<string> [delimiter]"
        split_at => "split at index", "<string> <index>"
        chars => "to char list", "<string>"
        words => "to word list", "<string>"
        words_quoted => "to word list, quoted as one", "<string>"
        lines => "to line list", "<string>"
        paragraphs => "to paragraph list", "<string>"
        concat => "join strings", "<string>..."

        // 修改操作
        insert => "insert string at index", "<string> <index> <string>"
        repeat => "repeat n times", "<string> <count>"
        replace => "replace all matches", "<string> <old> <new>"
        slice => "substring [start,end)", "<string> <start> [end]"
        rev => "reverse", "<string>"
        strip_prefix => "remove prefix", "<string> <prefix>"
        strip_suffix => "remove suffix", "<string> <suffix>"
        trim => "trim both ends", "<string>"
        trim_start => "trim start", "<string>"
        trim_end => "trim end", "<string>"
        lower => "to lowercase", "<string>"
        upper => "to uppercase", "<string>"
        title => "to title case", "<string>"

        // 高级操作
        max_len => "max line length", "<string>"
        grep => "lines matching substring", "<string> <substring>"
        strip_ansi => "remove ANSI codes", "<string>"
        escape => "escape control chars to \\n \\t \\xNN etc.", "<string>"
        unescape => "reverse of escape, parses \\n \\t \\xNN \\uXXXX", "<string>"

        // 格式化
        pad_start => "pad at start", "<string> <length> [pad_char=' ']"
        pad_end => "pad at end", "<string> <length> [pad_char=' ']"
        center => "pad both ends", "<string> <length> [pad_char=' ']"
        wrap => "wrap to width", "<string> <width>"

        // 样式
        href => "terminal hyperlink", "<url> <text>"
        bold => "bold", "<string>"
        dim => "dim", "<string>"
        italic => "italic", "<string>"
        underline => "underline", "<string>"
        blink => "blink", "<string>"
        invert => "invert fg/bg", "<string>"
        strike => "strikethrough", "<string>"

        // 标准颜色
        black => "black fg", "<string>"
        red => "red fg", "<string>"
        green => "green fg", "<string>"
        yellow => "yellow fg", "<string>"
        blue => "blue fg", "<string>"
        magenta => "magenta fg", "<string>"
        cyan => "cyan fg", "<string>"
        white => "white fg", "<string>"

        // 高级颜色
        clr => "256-color fg, code 0-255", "<string> <color_code>"
        clr_bg => "256-color bg, code 0-255", "<string> <color_code>"
        color => "true color fg. e.g. #ff0000, red, 255,0,0", "<string> <#hex|name|r,g,b>"
        color_bg => "true color bg. e.g. #ff0000, red, 255,0,0", "<string> <#hex|name|r,g,b>"
        colors => "list color names, or with swatches", "[swatches?]"
    })
}

// Basic Check Functions
fn is_empty(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_empty", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.is_empty()))
}

fn is_whitespace(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_whitespace", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.chars().all(|c| c.is_whitespace())))
}

fn is_ascii(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_ascii", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.chars().all(|c| c.is_ascii())))
}

fn is_ascii_control(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_ascii_control", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(
        text.chars().all(|c| c.is_ascii_control()),
    ))
}

fn is_ascii_punctuation(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_ascii_control", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(
        text.chars().all(|c| c.is_ascii_punctuation()),
    ))
}

fn is_ascii_digit(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_ascii_digit", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(
        text.chars().all(|c| c.is_ascii_digit()),
    ))
}

fn is_ascii_hexdigit(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_ascii_digit", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(
        text.chars().all(|c| c.is_ascii_hexdigit()),
    ))
}

fn is_alpha(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_alpha", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.chars().all(|c| c.is_alphabetic())))
}

fn is_alphanumeric(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_alphanumeric", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(
        text.chars().all(|c| c.is_alphanumeric()),
    ))
}

fn is_numeric(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_numeric", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.chars().all(|c| c.is_numeric())))
}

fn is_lower(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_lower", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.chars().all(|c| c.is_lowercase())))
}

fn is_upper(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_upper", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(text.chars().all(|c| c.is_uppercase())))
}

fn is_title(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_title", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let title = to_title_inner(text);
    Ok(Expression::Boolean(text == &title))
}

fn len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("len", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Integer(text.chars().count() as Int))
}
// Substring Check Functions
fn starts_with(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("starts_with", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let prefix = get_string_ref(&args[1], ctx)?;

    Ok(Expression::Boolean(text.starts_with(prefix)))
}

fn ends_with(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ends_with", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let suffix = get_string_ref(&args[1], ctx)?;

    Ok(Expression::Boolean(text.ends_with(suffix)))
}

fn contains(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("contains", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let substring = get_string_ref(&args[1], ctx)?;

    Ok(Expression::Boolean(text.contains(substring)))
}

// 查找子串首次出现的位置（按字符计数，与 len/substring 保持一致），未找到返回 None
fn position(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("position", &args, 2..=3, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let pattern = get_string_ref(&args[1], ctx)?;
    let start = if args.len() == 3 {
        get_integer_ref(&args[2], ctx)?.max(0) as usize
    } else {
        0
    };

    if pattern.is_empty() {
        return Ok(Expression::Integer(start.min(text.chars().count()) as Int));
    }

    let chars: Vec<char> = text.chars().collect();
    let pat_chars: Vec<char> = pattern.chars().collect();
    let plen = pat_chars.len();

    if start > chars.len() || plen > chars.len() {
        return Ok(Expression::None);
    }

    for i in start..=(chars.len() - plen) {
        if chars[i..i + plen] == pat_chars[..] {
            return Ok(Expression::Integer(i as Int));
        }
    }
    Ok(Expression::None)
}
// 按字符索引取值，支持负数索引（从末尾计算）
fn get(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let n = get_integer_ref(&args[1], ctx)?;

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let index = if n < 0 {
        let ni = len as i64 + n;
        if ni < 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::IndexOutOfBounds { index: n, len },
                ctx.clone(),
                0,
            ));
        }
        ni as usize
    } else {
        n as usize
    };

    chars
        .get(index)
        .map(|c| Expression::String(c.to_string()))
        .ok_or(RuntimeError::new(
            RuntimeErrorKind::IndexOutOfBounds { index: n, len },
            ctx.clone(),
            0,
        ))
}
// Splitting Operations
fn split(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("split", &args, 1..=2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    let ifs = env.get("IFS");
    let delimiter = if args.len() > 1 {
        let deli = get_string_ref(&args[1], ctx)?;
        Some(deli)
    } else {
        match (
            crate::runtime::ifs_contains(crate::runtime::IFS_STR, env),
            &ifs,
        ) {
            (true, Some(Expression::String(fs))) => Some(fs),
            _ => None,
        }
    };

    let parts: Vec<Expression> = match delimiter {
        Some(sep) => text
            .split(sep)
            .map(|s| Expression::String(s.to_string()))
            .collect(),
        _ => text
            .split_whitespace()
            .map(|s| Expression::String(s.to_string()))
            .collect(),
    };

    Ok(Expression::from(parts))
}

fn split_at(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("split_at", &args, 2, ctx)?;
    let mut it = args.into_iter();
    let t_expr = it.next().unwrap();
    let i_expr = it.next().unwrap();
    let text = get_string_arg(t_expr, ctx)?;
    let index = get_integer_ref(&i_expr, ctx)? as usize;

    if index > text.len() {
        return Ok(Expression::from(vec![
            Expression::String(text),
            Expression::String(String::new()),
        ]));
    }

    let (left, right) = text.split_at(index);
    Ok(Expression::from(vec![
        Expression::String(left.to_string()),
        Expression::String(right.to_string()),
    ]))
}

fn chars(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("chars", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    let chars = text
        .chars()
        .map(|c| Expression::String(c.to_string()))
        .collect::<Vec<Expression>>();
    Ok(Expression::from(chars))
}

fn words(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("words", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let words = text
        .split_whitespace()
        .map(|word| Expression::String(word.to_string()))
        .collect::<Vec<Expression>>();
    Ok(Expression::from(words))
}

fn words_quoted(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("words_quoted", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let re = QUOTED_RE
        .get_or_init(|| Regex::new(r#""((?:[^"\\]|\\.)*)"|'((?:[^'\\]|\\.)*)'|(\S+)"#).unwrap());
    let words = re
        .captures_iter(text)
        .filter_map(|cap| {
            cap.get(1)
                .or_else(|| cap.get(2))
                .or_else(|| cap.get(3))
                .map(|m| Expression::String(m.as_str().to_string()))
        })
        .collect::<Vec<Expression>>();

    Ok(Expression::from(words))
}

fn lines(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("lines", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    let lines = text
        .lines()
        .map(|line| Expression::String(line.to_string()))
        .collect::<Vec<Expression>>();
    Ok(Expression::from(lines))
}

fn paragraphs(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("paragraphs", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    let paragraphs = text
        .split("\n\n")
        .map(|para| Expression::String(para.to_string()))
        .collect::<Vec<Expression>>();
    Ok(Expression::from(paragraphs))
}
// Modification Operations
fn concat(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("concat", &args, 2.., ctx)?;

    let others = args
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .concat();

    Ok(Expression::from(others))
}
fn rev(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    top::rev(args, env, ctx)
}

fn repeat(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("repeat", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let count = get_integer_ref(&args[1], ctx)?.max(0) as usize;

    const MAX_RESULT_BYTES: usize = 1024 * 1024; // 1MB limit
    match text.len().checked_mul(count) {
        Some(total) if total <= MAX_RESULT_BYTES => Ok(Expression::String(text.repeat(count))),
        _ => Err(RuntimeError::common(
            format!(
                "string.repeat would produce a result larger than {}MB, refused",
                1
            )
            .into(),
            ctx.clone(),
            0,
        )),
    }
}

fn replace(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("replace", &args, 3, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let from = get_string_ref(&args[1], ctx)?;
    let to = get_string_ref(&args[2], ctx)?;

    Ok(Expression::String(text.replace(from, to)))
}

fn slice(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("substring", &args, 2..=3, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let start = get_integer_ref(&args[1], ctx)?;

    let start_idx = if start < 0 {
        (text.len() as i64 + start).max(0) as usize
    } else {
        start.min(text.len() as i64) as usize
    };

    let end_idx = if args.len() == 3 {
        let end = get_integer_ref(&args[2], ctx)?;
        if end < 0 {
            (text.len() as i64 + end).max(0) as usize
        } else {
            end.min(text.len() as i64) as usize
        }
    } else {
        text.len()
    };

    if start_idx >= end_idx || start_idx >= text.len() {
        return Ok(Expression::String(String::new()));
    }

    let result: String = text
        .chars()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .collect();
    Ok(Expression::String(result))
}

fn insert(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("insert", &args, 3, ctx)?;
    let mut it = args.into_iter();
    let arr = it.next().unwrap();
    let mut base = get_string_arg(arr, ctx)?;
    let idx = it.next().unwrap();
    let i = get_integer_arg(idx, ctx)?;
    let val = it.next().unwrap();

    if i as usize <= base.len() {
        base.insert_str(i as usize, &val.to_string());
        Ok(Expression::String(base))
    } else {
        Err(RuntimeError::new(
            RuntimeErrorKind::CustomError(
                format!("index {} out of bounds for insertion", i).into(),
            ),
            ctx.clone(),
            0,
        ))
    }
}

fn strip_prefix(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("remove_prefix", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let prefix = get_string_ref(&args[1], ctx)?;

    Ok(Expression::String(
        text.strip_prefix(prefix).unwrap_or(text).to_string(),
    ))
}

fn strip_suffix(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("remove_suffix", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let suffix = get_string_ref(&args[1], ctx)?;

    Ok(Expression::String(
        text.strip_suffix(suffix).unwrap_or(text).to_string(),
    ))
}

fn trim(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("trim", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::String(text.trim().to_string()))
}

fn trim_start(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("trim_start", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::String(text.trim_start().to_string()))
}

fn trim_end(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("trim_end", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::String(text.trim_end().to_string()))
}

fn lower(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("lower", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::String(text.to_lowercase()))
}

fn upper(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("upper", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::String(text.to_uppercase()))
}

fn to_title_inner(text: &str) -> String {
    let mut title = String::with_capacity(text.len());
    let mut capitalize = true;

    for c in text.chars() {
        if capitalize {
            title.extend(c.to_uppercase());
            capitalize = false;
        } else {
            title.extend(c.to_lowercase());
        }
        if c.is_whitespace() {
            capitalize = true;
        }
    }
    title
}

fn title(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("title", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let title = to_title_inner(text);
    Ok(Expression::String(title))
}

// Formatting Operations (continued)
fn pad_start(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("pad_start", &args, 2..=3, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let length = get_integer_ref(&args[1], ctx)?;
    let char = if args.len() > 2 {
        let c = get_string_ref(&args[2], ctx)?;
        c.chars().next().unwrap_or(' ')
    } else {
        ' '
    };

    pad_start_impl(length.max(0) as usize, char, text.to_string())
}

fn pad_end(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("pad_end", &args, 2..=3, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let length = get_integer_ref(&args[1], ctx)?;
    let char = if args.len() > 2 {
        let c = get_string_ref(&args[2], ctx)?;
        c.chars().next().unwrap_or(' ')
    } else {
        ' '
    };

    pad_end_impl(length.max(0) as usize, char, text.to_string())
}

fn center(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("center", &args, 2..=3, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let length = get_integer_ref(&args[1], ctx)?;
    let char = if args.len() > 2 {
        let c = get_string_ref(&args[2], ctx)?;
        c.chars().next().unwrap_or(' ')
    } else {
        ' '
    };

    center_impl(length.max(0) as usize, char, text.to_string())
}

fn wrap(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("wrap", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let columns = get_integer_ref(&args[1], ctx)?;
    Ok(textwrap::fill(text, columns as usize).into())
}

// Style Functions
fn href(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("href", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let url = get_string_ref(&args[1], ctx)?;

    Ok(format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text).into())
}

fn bold(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bold", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    Ok(format!("\x1b[1m{}\x1b[m\x1b[0m", text).into())
}

fn dim(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("dim", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    Ok(format!("\x1b[2m{}\x1b[m\x1b[0m", text).into())
}

fn italic(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("italic", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[3m{}\x1b[m\x1b[0m", text).into())
}

fn underline(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("underline", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[4m{}\x1b[m\x1b[0m", text).into())
}

fn blink(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("blink", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[5m{}\x1b[m\x1b[0m", text).into())
}

fn invert(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("invert", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[7m{}\x1b[m\x1b[0m", text).into())
}

fn strike(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("strike", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[9m{}\x1b[m\x1b[0m", text).into())
}
// Standard Color Functions
fn black(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("black", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[90m{}\x1b[m\x1b[0m", text).into())
}

fn red(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("red", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[91m{}\x1b[m\x1b[0m", text).into())
}

fn green(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("green", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[92m{}\x1b[m\x1b[0m", text).into())
}

fn yellow(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("yellow", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[93m{}\x1b[m\x1b[0m", text).into())
}

fn blue(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("blue", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[94m{}\x1b[m\x1b[0m", text).into())
}

fn magenta(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("magenta", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[95m{}\x1b[m\x1b[0m", text).into())
}

fn cyan(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("cyan", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[96m{}\x1b[m\x1b[0m", text).into())
}

fn white(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("white", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(format!("\x1b[97m{}\x1b[m\x1b[0m", text).into())
}
// Advanced Color Functions
fn clr(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("clr", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let color = get_integer_ref(&args[1], ctx)? as usize;

    Ok(format!("\x1b[38;5;{}m{}\x1b[m\x1b[0m", color, text).into())
}

fn clr_bg(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("clr_bg", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let color = get_integer_ref(&args[1], ctx)? as usize;
    Ok(format!("\x1b[48;5;{}m{}\x1b[m\x1b[0m", color, text).into())
}

fn color(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    true_color(args, false, env, ctx)
}

fn color_bg(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    true_color(args, true, env, ctx)
}

fn colors(
    args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if !args.is_empty() && !args[0].is_truthy() {
        Ok(Expression::from(
            COLOR_MAP
                .iter()
                .map(|(&k, _)| Expression::String(k.to_owned()))
                .collect::<Vec<_>>(),
        ))
    } else {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();

        for (i, (text, (r, g, b))) in COLOR_MAP.iter().enumerate() {
            let pad_len = 20 - text.len();
            let padding: String = std::iter::repeat_n(" ", pad_len).collect();
            write!(
                &mut stdout,
                "\x1b[48;2;{r};{g};{b}m     \x1b[m\x1b[0m \x1b[38;2;{r};{g};{b}m{text}\x1b[m\x1b[0m{padding}"
            ).unwrap();
            if i % 3 == 2 {
                writeln!(&mut stdout, "\n").unwrap();
            }
        }
        writeln!(&mut stdout).unwrap();
        Ok(Expression::None)
    }
}
// Helper Implementation Functions
fn pad_start_impl(len: usize, pad_ch: char, s: String) -> Result<Expression, RuntimeError> {
    if s.len() >= len {
        return Ok(Expression::String(s));
    }
    let pad_len = len - s.len();
    let padding: String = std::iter::repeat_n(pad_ch, pad_len).collect();
    Ok(Expression::String(format!("{padding}{s}")))
}

fn pad_end_impl(len: usize, pad_ch: char, s: String) -> Result<Expression, RuntimeError> {
    if s.len() >= len {
        return Ok(Expression::String(s));
    }
    let pad_len = len - s.len();
    let padding: String = std::iter::repeat_n(pad_ch, pad_len).collect();
    Ok(Expression::String(format!("{s}{padding}")))
}

fn center_impl(len: usize, pad_ch: char, s: String) -> Result<Expression, RuntimeError> {
    let total_pad = len - s.len();
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad;
    let left: String = std::iter::repeat_n(pad_ch, left_pad).collect();
    let right: String = std::iter::repeat_n(pad_ch, right_pad).collect();
    Ok(Expression::String(format!("{left}{s}{right}")))
}

fn true_color(
    args: Vec<Expression>,
    is_bg: bool,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("true_color", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let color_spec = get_string_ref(&args[1], ctx)?;

    let color_code = if let Some(hex) = color_spec.strip_prefix('#') {
        // Parse hex color
        true_color_by_hex(hex, is_bg, ctx)?
    } else {
        // Parse RGB values
        let parts: Vec<&str> = color_spec.split(',').collect();
        match parts.len() {
            1 => {
                // Parse name
                if let Some((r, g, b)) = COLOR_MAP.get(&color_spec.as_str()) {
                    format!("{};{};{}", r, g, b)
                } else {
                    return Err(RuntimeError::common(
                        "invalid color name".into(),
                        ctx.clone(),
                        0,
                    ));
                }
            }
            // Parse RGB
            3 => parts.join(";").to_string(),
            _ => {
                return Err(RuntimeError::common(
                    "invalid color format, expected hex or r,g,b".into(),
                    ctx.clone(),
                    0,
                ));
            }
        }
    };

    let prefix = if is_bg { "48" } else { "38" };
    Ok(format!("\x1b[{};2;{}m{}\x1b[m\x1b[0m", prefix, color_code, text).into())
}

fn max_len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("max_len", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;

    let max_width = text.lines().map(|line| line.len()).max().unwrap_or(0);

    Ok(Expression::Integer(max_width as Int))
}

fn grep(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("grep", &args, 2, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let pat: &str = get_string_ref(&args[1], ctx)?;

    let lines = text
        .lines()
        .filter(|x| x.contains(pat))
        .map(|line| Expression::String(line.to_string()))
        .collect::<Vec<Expression>>();
    Ok(Expression::from(lines))
}

fn to_safe(
    args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let str = args
        .into_iter()
        .next()
        .map_or("".to_string(), |exp| exp.to_string());
    Ok(Expression::StringSafe(str))
}

fn unescape(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("unescape", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::String(unescape_str(text)))
}

// 转义：将真实的控制字符/特殊字符转成可打印的 \n \t \\ 等序列
fn escape(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("escape", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\x07' => out.push_str("\\a"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\x0b' => out.push_str("\\v"),
            '\x1b' => out.push_str("\\e"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    Ok(Expression::String(out))
}
