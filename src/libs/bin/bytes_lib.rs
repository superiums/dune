use crate::libs::BuiltinInfo;
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
        contains, index_of, starts_with, ends_with,
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
        from => "create bytes from a string (utf8) or list of integers", "<string|list>"
        from_hex => "create bytes from a hex string", "<hex_string>"
        from_base64 => "create bytes from a base64 string", "<base64_string>"
        from_list => "create bytes from a list of integers(0-255)", "<list>"
        from_escaped => "create bytes from escaped text", "<string>"

        to_string => "convert bytes to a utf8 string (lossy)", "<bytes>"
        to_hex => "convert bytes to a hex string", "<bytes>"
        to_base64 => "convert bytes to a base64 string", "<bytes>"
        to_list => "convert bytes to a list of integers", "<bytes>"

        // 基本信息
        len => "get length of bytes", "<bytes>"
        is_empty => "check if bytes is empty", "<bytes>"

        // 查找
        contains => "check if bytes contains a sub-byte-sequence", "<bytes> <bytes>"
        index_of => "find index of a sub-byte-sequence, -1 if not found", "<bytes> <bytes>"
        starts_with => "check if bytes starts with a prefix", "<bytes> <bytes>"
        ends_with => "check if bytes ends with a suffix", "<bytes> <bytes>"

        // 切片/拼接
        slice => "get a slice of bytes by start,end index", "<bytes> <start> <end>"
        concat => "concat two bytes", "<bytes> <bytes>"
        repeat => "repeat bytes n times", "<bytes> <n>"

        // 结构修改
        push => "append a byte(int 0-255) to bytes, return new bytes", "<bytes> <int>"
        pop => "remove the last byte, return new bytes", "<bytes>"
        reverse => "reverse bytes", "<bytes>"

        // 分割
        split => "split bytes by a byte separator(int 0-255)", "<bytes> <int>"
    })
}

// ---------------- 构造/转换 ----------------

fn from(
    mut args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("from", &args, 1, ctx)?;
    let v = match args.pop().unwrap() {
        Expression::String(s) | Expression::Symbol(s) => s.into_bytes(),
        Expression::List(list) => list
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
            .collect::<Result<Vec<u8>, _>>()?,
        other => {
            return Err(RuntimeError::new(
                RuntimeErrorKind::TypeError {
                    expected: "String/List".into(),
                    found: other.type_name(),
                    sym: other.to_string(),
                },
                ctx.clone(),
                0,
            ));
        }
    };
    Ok(Expression::Bytes(v))
}

fn from_list(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    from(args, env, ctx)
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
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
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

fn index_of(
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
