use crate::{Environment, Expression, RuntimeError};
use std::{borrow::Cow, path::PathBuf};

// Helper functions

pub fn expand_home(path: &'_ str) -> Cow<'_, str> {
    if path.starts_with("~")
        && let Some(home_dir) = dirs::home_dir()
    {
        return Cow::Owned(path.replace("~", home_dir.to_string_lossy().as_ref()));
    }
    Cow::Borrowed(path)
}

pub fn get_std_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
pub fn get_current_path(env: &mut Environment) -> PathBuf {
    // for compaty of hot key binding, cwd was changed in another env
    // use slash cmd insteadof key binding
    // get_std_cwd()
    env.get("PWD").map_or(get_std_cwd(), |v| match v {
        Expression::String(s) => PathBuf::from(s),
        s => PathBuf::from(s.to_string()),
    })
}
pub fn get_current_path_string(env: &mut Environment) -> String {
    env.get("PWD").map_or(
        std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        |v| match v {
            Expression::String(s) => s,
            s => s.to_string(),
        },
    )
}

pub fn join_current_path(path: &str, env: &mut Environment) -> PathBuf {
    get_current_path(env).join(path)
}
pub fn abs(path: &str, env: &mut Environment) -> PathBuf {
    if path.starts_with("~") {
        return PathBuf::from(expand_home(path).as_ref());
    }

    join_current_path(path, env)
}
pub fn abs_script(path: &str, env: &mut Environment) -> PathBuf {
    if path.starts_with("~") {
        return PathBuf::from(expand_home(path).as_ref());
    }

    let base = match env.get("SCRIPT") {
        Some(Expression::String(s)) => match PathBuf::from(s).parent() {
            None => get_current_path(env),
            Some(p) if p.to_string_lossy() == "" => get_current_path(env),
            Some(p) => p.to_path_buf(),
        },
        _ => get_current_path(env),
    };
    base.join(path)
}
pub fn abs_check(path: &str, env: &mut Environment) -> Result<PathBuf, RuntimeError> {
    let abs = abs(path, env);
    if abs.exists() {
        return Ok(abs);
    }
    Err(RuntimeError::common(
        "Entry not found".into(),
        Expression::String(path.to_string()),
        0,
    ))
}
pub fn canon(p: &str, env: &mut Environment) -> Result<PathBuf, RuntimeError> {
    let path = abs(p, env);
    dunce::canonicalize(&path).map_err(|e| {
        RuntimeError::from_io_error(e, "canon".into(), Expression::String(p.to_string()), 0)
    })
}

/// 自定义转义处理器，替换 snailquote::unescape
/// 输入为去掉外层引号后的裸字符串内容
/// 对于无法识别的转义序列，保留 \X 原样（宽容模式）
pub fn unescape_str(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }

        match chars.next() {
            None => {
                // 末尾孤立的反斜杠，保留
                result.push('\\');
            }
            Some(next) => match next {
                // 标准转义
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '0' => result.push('\0'),
                'a' => result.push('\x07'),       // 响铃
                'b' => result.push('\x08'),       // 退格
                'f' => result.push('\x0c'),       // 换页
                'v' => result.push('\x0b'),       // 垂直制表符
                'e' | 'E' => result.push('\x1b'), // ESC（统一处理，不再需要预处理）
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                '`' => result.push('`'),

                // \xNN 十六进制字节
                'x' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() == 2 {
                        if let Ok(n) = u8::from_str_radix(&hex, 16) {
                            // 作为 Unicode 标量值处理，支持 ASCII 范围
                            result.push(n as char);
                        } else {
                            result.push('\\');
                            result.push('x');
                            result.push_str(&hex);
                        }
                    } else {
                        result.push('\\');
                        result.push('x');
                        result.push_str(&hex);
                    }
                }

                // \033 \007 等八进制（3位）
                '0'..='7' => {
                    let mut oct = String::with_capacity(3);
                    oct.push(next);
                    for _ in 0..2 {
                        if matches!(chars.peek(), Some('0'..='7')) {
                            oct.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if let Ok(n) = u32::from_str_radix(&oct, 8) {
                        if let Some(ch) = char::from_u32(n) {
                            result.push(ch);
                        } else {
                            result.push('\\');
                            result.push_str(&oct);
                        }
                    } else {
                        result.push('\\');
                        result.push_str(&oct);
                    }
                }

                // \u{NNNN} 或 \uNNNN（4位）
                'u' => {
                    if chars.peek() == Some(&'{') {
                        chars.next(); // consume '{'
                        let hex: String = chars.by_ref().take_while(|&c| c != '}').collect();
                        match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                            Some(ch) => result.push(ch),
                            None => {
                                result.push_str("\\u{");
                                result.push_str(&hex);
                                result.push('}');
                            }
                        }
                    } else {
                        let hex: String = chars.by_ref().take(4).collect();
                        match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                            Some(ch) => result.push(ch),
                            None => {
                                result.push_str("\\u");
                                result.push_str(&hex);
                            }
                        }
                    }
                }

                // \UNNNNNNNN（8位 Unicode）
                'U' => {
                    let hex: String = chars.by_ref().take(8).collect();
                    match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        Some(ch) => result.push(ch),
                        None => {
                            result.push_str("\\U");
                            result.push_str(&hex);
                        }
                    }
                }

                // 无法识别的转义序列：保留原始 \X
                other => {
                    result.push('\\');
                    result.push(other);
                }
            },
        }
    }

    result
}

/// 字节字面量专用转义处理器：输出 Vec<u8> 而非 String
/// \xNN 表示原始字节值（可以是任意 0x00-0xFF，不做 UTF-8 编码）
/// 其它转义（\n \t \uXXXX 等）按其对应字符的 UTF-8 编码写入
pub fn unescape_bytes(s: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\\' {
            let mut buf = [0u8; 4];
            result.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            continue;
        }

        match chars.next() {
            None => result.push(b'\\'),
            Some(next) => match next {
                'n' => result.push(b'\n'),
                'r' => result.push(b'\r'),
                't' => result.push(b'\t'),
                '0' => result.push(0),
                'a' => result.push(0x07),
                'b' => result.push(0x08),
                'f' => result.push(0x0c),
                'v' => result.push(0x0b),
                'e' | 'E' => result.push(0x1b),
                '\\' => result.push(b'\\'),
                '"' => result.push(b'"'),
                '\'' => result.push(b'\''),
                '`' => result.push(b'`'),

                // \xNN -> 原始字节，不经过 UTF-8 编码
                'x' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    match u8::from_str_radix(&hex, 16) {
                        Ok(n) => result.push(n),
                        Err(_) => {
                            result.push(b'\\');
                            result.push(b'x');
                            result.extend_from_slice(hex.as_bytes());
                        }
                    }
                }

                // \033 八进制（3位）——同样直接作为字节值，而非 char::from_u32
                '0'..='7' => {
                    let mut oct = String::with_capacity(3);
                    oct.push(next);
                    for _ in 0..2 {
                        if matches!(chars.peek(), Some('0'..='7')) {
                            oct.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    match u32::from_str_radix(&oct, 8) {
                        Ok(n) if n <= 0xFF => result.push(n as u8),
                        _ => {
                            result.push(b'\\');
                            result.extend_from_slice(oct.as_bytes());
                        }
                    }
                }

                // \u{...} / \uXXXX / \UXXXXXXXX 表示 Unicode 字符，仍按 UTF-8 编码写入字节流
                'u' => {
                    let hex: String = if chars.peek() == Some(&'{') {
                        chars.next();
                        chars.by_ref().take_while(|&c| c != '}').collect()
                    } else {
                        chars.by_ref().take(4).collect()
                    };
                    match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        Some(ch) => {
                            let mut buf = [0u8; 4];
                            result.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        None => {
                            result.push(b'\\');
                            result.push(b'u');
                            result.extend_from_slice(hex.as_bytes());
                        }
                    }
                }
                'U' => {
                    let hex: String = chars.by_ref().take(8).collect();
                    match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        Some(ch) => {
                            let mut buf = [0u8; 4];
                            result.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        None => {
                            result.push(b'\\');
                            result.push(b'U');
                            result.extend_from_slice(hex.as_bytes());
                        }
                    }
                }

                other => {
                    result.push(b'\\');
                    let mut buf = [0u8; 4];
                    result.extend_from_slice(other.encode_utf8(&mut buf).as_bytes());
                }
            },
        }
    }

    result
}
