use crate::libs::BuiltinInfo;
use crate::libs::bin::list_lib::get_list_ref;
use crate::libs::helper::*;
use crate::libs::lazy_module::LazyModule;
use crate::utils::unescape_bytes;
use crate::{Environment, Expression, Int, RuntimeError, RuntimeErrorKind};
use crate::{reg_info, reg_lazy};
use std::collections::BTreeMap;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 构造/转换
        from, from_hex, from_base64, from_list, from_escaped,
        to_string, to_hex, to_base64, to_list,
        // 基本信息
        len, is_empty,
        // 查找
        contains, position, starts_with, ends_with,
        // 切片/拼接
        slice, concat, repeat,
        // 结构修改
        push, pop, reverse,
        // 分割
        split,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 构造/转换
        from => "bytes from utf8 string", "<string>"
        from_hex => "bytes from hex string, '0x' prefix ok", "<hex_string>"
        from_base64 => "bytes from base64 string", "<base64_string>"
        from_list => "bytes from int(0-255) list", "<list>"
        from_escaped => "bytes from escaped text. e.g. '\\n\\x41'", "<string>"

        to_string => "to utf8 string (lossy)", "<bytes>"
        to_hex => "to hex string", "<bytes>"
        to_base64 => "to base64 string", "<bytes>"
        to_list => "to list of int(0-255)", "<bytes>"

        // 基本信息
        len => "byte length", "<bytes>"
        is_empty => "is empty?", "<bytes>"

        // 查找
        contains => "contains sub-sequence?", "<bytes> <bytes>"
        position => "index of sub-sequence, -1 if absent", "<bytes> <bytes>"
        starts_with => "starts with prefix?", "<bytes> <bytes>"
        ends_with => "ends with suffix?", "<bytes> <bytes>"

        // 切片/拼接
        slice => "sub-bytes [start,end)", "<bytes> <start> <end>"
        concat => "concat two bytes", "<bytes> <bytes>"
        repeat => "repeat n times", "<bytes> <n>"

        // 结构修改
        push => "append byte(0-255), returns new bytes", "<bytes> <int>"
        pop => "drop last byte, returns new bytes", "<bytes>"
        reverse => "reverse bytes", "<bytes>"

        // 分割
        split => "split by byte value(0-255)", "<bytes> <int>"
    })
}

// ---------------- 构造/转换 ----------------

fn from(
    mut args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from", &args, 1, ctx)?;
    let s = get_string_arg(args.pop().unwrap(), ctx)?;
    Ok(Expression::Bytes(s.into_bytes()))
}

fn from_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_list", &args, 1, ctx)?;
    let list = get_list_ref(&args[0], ctx)?;
    let v = list
        .iter()
        .map(|item| match item {
            Expression::Integer(i) if (0..=255).contains(i) => Ok(*i as u8),
            other => Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "Integer(0-255)".into(),
                    found: other.type_name(),
                    sym: other.to_string(),
                },
                ctx.clone(),
                0,
            )),
        })
        .collect::<Result<Vec<u8>, _>>()?;

    Ok(Expression::Bytes(v))
}

fn from_hex(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_hex", &args, 1, ctx)?;
    let s = get_string_ref(&args[0], ctx)?;
    let s = s.strip_prefix("0x").unwrap_or(s.as_str());
    if s.len() % 2 != 0 {
        return Err(RuntimeError::common(
            "invalid hex string: odd length".into(),
            ctx.clone(),
            0,
        ));
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let chars: Vec<char> = s.chars().collect();
    for pair in chars.chunks(2) {
        let byte_str: String = pair.iter().collect();
        let b = u8::from_str_radix(&byte_str, 16).map_err(|e| {
            RuntimeError::common(format!("invalid hex string: {e}").into(), ctx.clone(), 0)
        })?;
        bytes.push(b);
    }
    Ok(Expression::Bytes(bytes))
}

fn to_hex(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_hex", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let s: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    Ok(Expression::String(s))
}

const B64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn to_base64(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_base64", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        out.push(B64_TABLE[(b0 >> 2) as usize] as char);
        out.push(B64_TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64_TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64_TABLE[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    Ok(Expression::String(out))
}

fn from_base64(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_base64", &args, 1, ctx)?;
    let s = get_string_ref(&args[0], ctx)?;
    let s = s.trim_end_matches('=');

    fn b64_val(c: u8, ctx: &Expression) -> Result<u8, RuntimeError> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(RuntimeError::common(
                "invalid base64 character".into(),
                ctx.clone(),
                0,
            )),
        }
    }

    let chars = s.as_bytes();
    let mut out = Vec::with_capacity(chars.len() / 4 * 3);
    for chunk in chars.chunks(4) {
        let vals: Vec<u8> = chunk
            .iter()
            .map(|&c| b64_val(c, ctx))
            .collect::<Result<Vec<_>, _>>()?;
        let v0 = vals[0];
        let v1 = *vals.get(1).unwrap_or(&0);
        let v2 = vals.get(2).copied();
        let v3 = vals.get(3).copied();

        out.push((v0 << 2) | (v1 >> 4));
        if let Some(v2) = v2 {
            out.push((v1 << 4) | (v2 >> 2));
        }
        if let Some(v3) = v3 {
            out.push((v2.unwrap_or(0) << 6) | v3);
        }
    }
    Ok(Expression::Bytes(out))
}

fn to_string(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_string", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    Ok(Expression::String(
        String::from_utf8_lossy(bytes).to_string(),
    ))
}

fn to_list(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_list", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let list = bytes
        .iter()
        .map(|b| Expression::Integer(*b as Int))
        .collect::<Vec<_>>();
    Ok(Expression::from(list))
}

// ---------------- 基本信息 ----------------

fn len(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("len", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    Ok(Expression::Integer(bytes.len() as Int))
}

fn is_empty(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_empty", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    Ok(Expression::Boolean(bytes.is_empty()))
}

// ---------------- 查找 ----------------

fn contains(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("contains", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let sub = get_bytes_ref(&args[1], ctx)?;
    Ok(Expression::Boolean(
        find_subslice(bytes, sub).is_some() || sub.is_empty(),
    ))
}

fn position(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("index_of", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let sub = get_bytes_ref(&args[1], ctx)?;
    Ok(Expression::Integer(
        find_subslice(bytes, sub).map(|i| i as Int).unwrap_or(-1),
    ))
}

fn starts_with(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("starts_with", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let prefix = get_bytes_ref(&args[1], ctx)?;
    Ok(Expression::Boolean(bytes.starts_with(prefix.as_slice())))
}

fn ends_with(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ends_with", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let suffix = get_bytes_ref(&args[1], ctx)?;
    Ok(Expression::Boolean(bytes.ends_with(suffix.as_slice())))
}

// ---------------- 切片/拼接 ----------------

fn slice(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("slice", &args, 3, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let start = get_integer_ref(&args[1], ctx)?.max(0) as usize;
    let end = (get_integer_ref(&args[2], ctx)?.max(0) as usize).min(bytes.len());
    if start > end {
        return Ok(Expression::Bytes(vec![]));
    }
    Ok(Expression::Bytes(bytes[start..end].to_vec()))
}

fn concat(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("concat", &args, 2, ctx)?;
    let a = get_bytes_ref(&args[0], ctx)?;
    let b = get_bytes_ref(&args[1], ctx)?;
    let mut v = a.clone();
    v.extend_from_slice(b);
    Ok(Expression::Bytes(v))
}

fn repeat(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("repeat", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let n = get_integer_ref(&args[1], ctx)?.max(0) as usize;
    Ok(Expression::Bytes(bytes.repeat(n)))
}

// ---------------- 结构修改 ----------------

fn push(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("push", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let n = get_integer_ref(&args[1], ctx)?;
    if !(0..=255).contains(&n) {
        return Err(RuntimeError::common(
            "push requires an integer in range 0-255".into(),
            ctx.clone(),
            0,
        ));
    }
    let mut v = bytes.clone();
    v.push(n as u8);
    Ok(Expression::Bytes(v))
}

fn pop(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("pop", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let mut v = bytes.clone();
    v.pop();
    Ok(Expression::Bytes(v))
}

fn reverse(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("reverse", &args, 1, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    Ok(Expression::Bytes(bytes.iter().rev().cloned().collect()))
}

// ---------------- 分割 ----------------

fn split(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("split", &args, 2, ctx)?;
    let bytes = get_bytes_ref(&args[0], ctx)?;
    let sep = get_integer_ref(&args[1], ctx)?;
    if !(0..=255).contains(&sep) {
        return Err(RuntimeError::common(
            "split requires an integer separator in range 0-255".into(),
            ctx.clone(),
            0,
        ));
    }
    let parts = bytes
        .split(|b| *b == sep as u8)
        .map(|p| Expression::Bytes(p.to_vec()))
        .collect::<Vec<_>>();
    Ok(Expression::from(parts))
}

fn from_escaped(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from_escaped", &args, 1, ctx)?;
    let text = get_string_ref(&args[0], ctx)?;
    Ok(Expression::Bytes(unescape_bytes(text)))
}

// ---------------- 辅助函数 ----------------

fn get_bytes_ref<'a>(expr: &'a Expression, ctx: &Expression) -> Result<&'a Vec<u8>, RuntimeError> {
    match expr {
        Expression::Bytes(b) => Ok(b),
        e => Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "Bytes".into(),
                found: e.type_name(),
                sym: e.to_string(),
            },
            ctx.clone(),
            0,
        )),
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > haystack.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
