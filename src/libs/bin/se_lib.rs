use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::OnceLock,
};

use regex_lite::Regex;

use crate::{
    Environment, Expression, RuntimeError, RuntimeErrorKind,
    eval::State,
    libs::{
        BuiltinInfo, SelfExpandFunc,
        helper::{
            check_args_len, check_exact_args_len, get_integer_arg, get_string_arg, get_table_arg,
        },
    },
    reg_info,
};
static NAMED_RE: OnceLock<Regex> = OnceLock::new();
static POSITION_RE: OnceLock<Regex> = OnceLock::new();

pub fn regist_se() -> HashMap<&'static str, SelfExpandFunc> {
    let mut module: HashMap<&'static str, SelfExpandFunc> = HashMap::new();
    module.insert("format", format);
    module.insert("where", r#where);
    module.insert("repeat", repeat);
    module.insert("assert", assert);
    module.insert("when", when);
    module.insert("debug", debug);
    module.insert("ddebug", ddebug);
    module.insert("symof", symof);
    module.insert("typeof", r#typeof);
    module.insert("set_root", set_root);
    module.insert("unset_root", unset_root);
    module.insert("get_local", get_local);
    module.insert("get_env", get_env);
    module.insert("get_var", get_var);
    module.insert("quote", quote);
    module
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
      // debug
      when => "conditional execute", "<condition> <execute>"
      assert => "throw if not equal/truthy", "<expr> [expr] [message]"
      debug => "eval & show expr,type,value(debug fmt)", "<args>..."
      ddebug => "eval & show expr,type,value(pretty fmt)", "<args>..."
      symof => "type name before eval", "<value>"
      typeof => "type name after eval", "<value>"
      quote => "quote expr, eval later", "<expr>"

      // Data manipulation
      format => "fmt string. {name}/{} for named/positional, :spec for align.\n\te.g. format '{:0>5}' 3 -> 00003", "<template> <args>..."
      where => "filter table rows. NR/<col_name> injected. e.g. where t (NR>1 and col>0)", "<table> <condition>"

      // Execution control
      repeat => "eval expr n times, collect non-None results", "<expr> <n>"

      // env
      set_root => "define var in root env", "<var> <val>"
      unset_root => "undefine var in root env", "<var>"
      get_local => "get local var value", "<var>"
      get_env => "get var from env", "<var>"
      get_var => "get var, local first then env", "<var>"
    })
}

// args should be lazy evaled.
fn r#where(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("where", args, 2, ctx)?;
    let data_evaled = args[0].eval_mut(state, env, 0)?;
    let data = get_table_arg(data_evaled, ctx)?;

    let is_last_local = state.contains(State::IN_LOCAL);
    let last_local_vars = if is_last_local {
        Some(state.get_local_vars())
    } else {
        None
    };
    state.set(State::IN_LOCAL);

    let predicate = |nr: usize, row: &[Expression]| -> bool {
        state.set_local_var("NR".to_string(), Expression::Integer(nr as i64));
        for (nf, cell) in row.iter().enumerate() {
            let name = data
                .headers()
                .get(nf)
                .map_or("unkown".to_string(), |x| x.to_string());
            state.set_local_var(name, cell.clone());
            // state.set_local_var("NF".to_string(), Expression::Integer(nf as i64));
        }
        match args[1].eval_mut(state, env, 0) {
            Ok(x) => x.is_truthy(),
            _ => false,
        }
    };

    let filtered = data.filter_rows(predicate);

    if is_last_local {
        state.set_local_vars(last_local_vars.unwrap());
    } else {
        state.clear_local_var();
        state.clear(State::IN_LOCAL);
    }

    Ok(Expression::Table(filtered))
}

// args should be lazy evaled
/// pipe action NOT supported
fn repeat(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("repeat", args, 2, ctx)?;
    let n = get_integer_arg(args[1].eval_mut(state, env, 0)?, ctx)?;
    let results = (0..n)
        .map(|_| args[0].eval_with_assign(state, env))
        .collect::<Result<Vec<_>, _>>()?;
    if results.iter().any(|x| x != &Expression::None) {
        Ok(Expression::from(results))
    } else {
        Ok(Expression::None)
    }
}

fn assert(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("assert", &args, 1..=3, ctx)?;

    let a1 = args[0].eval_with_assign(state, env)?;
    let fail = {
        if args.len() > 1 {
            let a2 = args[1].eval_with_assign(state, env)?;
            a1 != a2
        } else {
            !a1.is_truthy()
        }
    };

    if fail {
        let message = if args.len() > 2 {
            args[2].eval_with_assign(state, env)?.to_string()
        } else {
            "assertion failed".to_string()
        };

        return Err(RuntimeError::new(
            RuntimeErrorKind::CustomError(message.into()),
            ctx.clone(),
            0,
        ));
    }

    Ok(Expression::None)
}

fn when(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("when", &args, 2, ctx)?;

    if args[0].eval_with_assign(state, env)?.is_truthy() {
        return args[1].eval_with_assign(state, env);
    }

    Ok(Expression::None)
}

// args lazy
fn debug(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let mut results = Vec::new();
    for x in args.iter() {
        let expr_repr = format!("{x:?}");
        let y = x.eval_with_assign(state, env);
        let mut map = BTreeMap::new();
        map.insert("expr".to_string(), Expression::String(expr_repr));
        match y {
            Ok(r) => {
                map.insert(
                    "type".to_string(),
                    Expression::String(r.type_name().to_string()),
                );
                map.insert("value".to_string(), r);
            }
            Err(e) => {
                map.insert("type".to_string(), Expression::String("Err".to_string()));
                map.insert("value".to_string(), Expression::String(e.kind.to_string()));
            }
        };
        results.push(Expression::from(map));
    }
    Ok(Expression::from(results))
}

// args lazy
fn ddebug(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let mut results = Vec::new();
    for x in args.iter() {
        let expr_repr = format!("{x:#}");
        let y = x.eval_with_assign(state, env);
        let mut map = BTreeMap::new();
        map.insert("expr".to_string(), Expression::String(expr_repr));
        match y {
            Ok(r) => {
                map.insert(
                    "type".to_string(),
                    Expression::String(r.type_name().to_string()),
                );
                map.insert("value".to_string(), r);
            }
            Err(e) => {
                map.insert("type".to_string(), Expression::String("Err".to_string()));
                map.insert("value".to_string(), Expression::String(e.kind.to_string()));
            }
        };
        results.push(Expression::from(map));
    }
    Ok(Expression::from(results))
}

fn symof(
    args: &[Expression],
    _env: &mut Environment,
    _state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("symof", &args, 1, ctx)?;
    let t = args[0].type_name();
    Ok(Expression::from(t))
}

// arg lazy
fn r#typeof(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("typeof", args, 1, ctx)?;
    match args[0].eval_with_assign(state, env) {
        Ok(r) => Ok(Expression::from(r.type_name())),
        _ => Ok(Expression::String("Err".to_string())),
    }
}

/// 解析格式说明符，返回 (填充字符, 对齐方向, 宽度)
/// 支持：
///   "10"    → (' ', '<', 10)  仅宽度，默认左对齐
///   "<10"   → (' ', '<', 10)  左对齐
///   ">10"   → (' ', '>', 10)  右对齐
///   "^10"   → (' ', '^', 10)  居中
///   "0>10"  → ('0', '>', 10)  右对齐，0填充
///   "*^5"   → ('*', '^', 5)   居中，*填充
fn parse_format_spec(spec: &str) -> (char, char, usize) {
    if spec.is_empty() {
        return (' ', '<', 0);
    }
    let chars: Vec<char> = spec.chars().collect();

    // [填充字符][对齐方向][宽度]：第二个字符是 < > ^
    if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^') {
        let width: usize = chars[2..].iter().collect::<String>().parse().unwrap_or(0);
        return (chars[0], chars[1], width);
    }

    // [对齐方向][宽度]：第一个字符是 < > ^
    if matches!(chars[0], '<' | '>' | '^') {
        let width: usize = chars[1..].iter().collect::<String>().parse().unwrap_or(0);
        return (' ', chars[0], width);
    }

    // 只有宽度
    (' ', '<', spec.parse().unwrap_or(0))
}

/// 应用对齐，使用字符数（chars().count()）而非字节数，支持中文等多字节字符
fn apply_align(s: &str, pad_ch: char, align: char, width: usize) -> String {
    if width == 0 {
        return s.to_string();
    }
    let char_len = s.chars().count();
    if char_len >= width {
        return s.to_string();
    }
    let pad = width - char_len;
    let make_pad = |n: usize| -> String { std::iter::repeat_n(pad_ch, n).collect() };
    match align {
        '>' => format!("{}{s}", make_pad(pad)),
        '^' => {
            let left = pad / 2;
            format!("{}{s}{}", make_pad(left), make_pad(pad - left))
        }
        _ => format!("{s}{}", make_pad(pad)), // '<' 或默认
    }
}

/// format need template to be first arg,
/// but pipe alwasy takes 1st place.
/// so we need to adjust it auto.
/// format need _ to self expand
fn format(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("format", args, 1.., ctx)?;
    let data0 = args[0].eval_mut(state, env, 0)?;

    if args.len() == 1 {
        return Ok(Expression::String(data0.to_string()));
    }

    let data1 = args[1].eval_mut(state, env, 0)?;

    let (template, data_first) = match args[0] {
        Expression::Blank => {
            let template = get_string_arg(data1, ctx)?;
            (template, data0)
        }
        _ => {
            let template = get_string_arg(data0, ctx)?;
            (template, data1)
        }
    };

    // 第一步：替换命名参数 {name} 或 {name:spec}
    // 注意：有可选捕获组，不能用 extract()，改用 cap.get(n)
    let named_re = NAMED_RE.get_or_init(|| Regex::new(r"\{(\w+)(?::([^}]*))?\}").unwrap());
    let mut result = String::new();
    let mut last_end = 0;
    for cap in named_re.captures_iter(&template) {
        let m = cap.get(0).unwrap();
        result.push_str(&template[last_end..m.start()]);
        let var = cap.get(1).unwrap().as_str();
        let spec = cap.get(2).map_or("", |m| m.as_str());
        let value = ctx.handle_variable(var, false, state, env, 0)?.to_string();
        let (pad_ch, align, width) = parse_format_spec(spec);
        result.push_str(&apply_align(&value, pad_ch, align, width));
        last_end = m.end();
    }
    result.push_str(&template[last_end..]);

    // 第二步：替换位置参数 {} 或 {:spec}
    // 收集所有位置参数：data_first 排第一，args[2..] 依次跟随
    let mut pos_args: Vec<String> = vec![data_first.to_string()];
    for arg in args.iter().skip(2) {
        pos_args.push(arg.eval_mut(state, env, 0)?.to_string());
    }

    let pos_re = POSITION_RE.get_or_init(|| Regex::new(r"\{(?::([^}]*))?\}").unwrap());
    let template2 = result;
    let mut result = String::new();
    let mut last_end = 0;
    let mut pos_idx = 0;
    for cap in pos_re.captures_iter(&template2) {
        let m = cap.get(0).unwrap();
        result.push_str(&template2[last_end..m.start()]);
        let spec = cap.get(1).map_or("", |m| m.as_str());
        if let Some(val) = pos_args.get(pos_idx) {
            let (pad_ch, align, width) = parse_format_spec(spec);
            result.push_str(&apply_align(val, pad_ch, align, width));
            pos_idx += 1;
        } else {
            result.push_str(m.as_str()); // 参数不足时保留原占位符
        }
        last_end = m.end();
    }
    result.push_str(&template2[last_end..]);

    Ok(Expression::String(result))
}

// lazy arg
pub fn set_root(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("set_root", args, 2, ctx)?;
    let name = args[0].to_string();
    let expr = args[1].eval_with_assign(state, env)?;
    env.define_in_root(&name, expr);
    Ok(Expression::None)
}

// lazy arg
pub fn unset_root(
    args: &[Expression],
    env: &mut Environment,
    _state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("unset_root", args, 1, ctx)?;
    let name = args[0].to_string();
    env.undefine_in_root(&name);
    Ok(Expression::None)
}

pub fn get_local(
    args: &[Expression],
    _env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get_local", args, 1, ctx)?;
    let name = args[0].to_string();
    let r = state
        .get_local_var(&name)
        .cloned()
        .unwrap_or(Expression::None);
    Ok(r)
}

pub fn get_env(
    args: &[Expression],
    env: &mut Environment,
    _state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get_env", args, 1, ctx)?;
    let name = args[0].to_string();
    let r = env.get(&name).unwrap_or(Expression::None);
    Ok(r)
}

pub fn get_var(
    args: &[Expression],
    env: &mut Environment,
    state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("get_var", args, 1, ctx)?;
    let name = args[0].to_string();

    if let Some(local_val) = state.get_local_var(&name) {
        return Ok(local_val.clone());
    }

    let r = env.get(&name).unwrap_or(Expression::None);
    Ok(r)
}

fn quote(
    args: &[Expression],
    _env: &mut Environment,
    _state: &mut State,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("quote", &args, 1, ctx)?;
    Ok(Expression::Quote(Rc::new(args[0].clone())))
}
