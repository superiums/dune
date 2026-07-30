use crate::{Environment, Expression, RuntimeError, RuntimeErrorKind};
use rand::distr::SampleString;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng, prelude::SliceRandom};
use std::cell::RefCell;

use crate::libs::BuiltinInfo;
use crate::libs::helper::{check_exact_args_len, get_integer_ref};
use crate::libs::lazy_module::LazyModule;
use crate::{reg_info, reg_lazy};
use std::collections::BTreeMap;

thread_local! {
    // 默认 None，表示使用系统熵源（ThreadRng）；调用 rand.seed 后固定为可复现序列
    static SEEDED_RNG: RefCell<Option<StdRng>> = RefCell::new(None);
}

// 统一的随机源获取入口：所有随机函数都应通过这里拿 rng，
// 这样 rand.seed() 才能真正影响后续所有随机调用
fn with_rng<R>(f: impl FnOnce(&mut dyn Rng) -> R) -> R {
    SEEDED_RNG.with(|cell| {
        let mut guard = cell.borrow_mut();
        if let Some(rng) = guard.as_mut() {
            f(rng)
        } else {
            drop(guard);
            let mut rng = rand::rng();
            f(&mut rng)
        }
    })
}

fn get_float_ref(expr: &Expression, ctx: &Expression) -> Result<f64, RuntimeError> {
    match expr {
        Expression::Integer(i) => Ok(*i as f64),
        Expression::Float(f) => Ok(*f),
        e => Err(RuntimeError::new(
            RuntimeErrorKind::TypeError {
                expected: "Integer/Float".into(),
                found: e.type_name(),
                sym: e.to_string(),
            },
            ctx.clone(),
            0,
        )),
    }
}

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 可复现性
        seed,
        // 概率函数
        chance, ratio,
        // 随机字符串生成
        alpha, alphanum,
        // 数值随机
        int, float,
        // 集合操作
        choose, shuffle, sample,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 可复现性
        seed => "seed generator for reproducible sequence", "<integer>"

        // 概率函数
        chance => "random bool with probability p", "[p=0.5]"
        ratio => "random bool with probability num/den", "<num> <den>"

        // 随机字符串生成
        alpha => "random alphabetic char(s)", "[len=1]"
        alphanum => "random alphanumeric char(s)", "[len=1]"

        // 数值随机
        int => "random integer. no args: any i64; 1 arg: [0,max]; 2 args: [min,max)", "[min] [max]"
        float => "random float. no args: [0,1); 2 args: [min,max)", "[min] [max]"

        // 集合操作
        choose => "pick random item", "<list>"
        shuffle => "shuffle order, returns new list", "<list>"
        sample => "pick n distinct items, no replacement", "<list> <n>"
    })
}

// ===== 可复现性 =====
fn seed(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("seed", &args, 1, ctx)?;
    let n = get_integer_ref(&args[0], ctx)?;
    SEEDED_RNG.with(|cell| {
        *cell.borrow_mut() = Some(StdRng::seed_from_u64(n as u64));
    });
    Ok(Expression::None)
}

// ===== 概率函数（原 ratio 拆分而来） =====
fn chance(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    match args.len() {
        0 => {
            let b = with_rng(|rng| rng.random_bool(0.5));
            Ok(Expression::Boolean(b))
        }
        1 => {
            if let Expression::Float(f) = &args[0] {
                let b = with_rng(|rng| rng.random_bool(*f));
                Ok(Expression::Boolean(b))
            } else {
                Err(RuntimeError::common(
                    "rand.chance expected float probability".into(),
                    ctx.clone(),
                    0,
                ))
            }
        }
        _ => Err(RuntimeError::common(
            "rand.chance expected 0 or 1 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

fn ratio(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ratio", &args, 2, ctx)?;
    match (&args[0], &args[1]) {
        (Expression::Integer(numerator), Expression::Integer(denominator)) => {
            let b = with_rng(|rng| rng.random_ratio(*numerator as u32, *denominator as u32));
            Ok(Expression::Boolean(b))
        }
        (l, h) => Err(RuntimeError::common(
            format!("rand.ratio expected two integers, but got {l} and {h}").into(),
            ctx.clone(),
            0,
        )),
    }
}

// ===== 随机字符串生成（改为走 with_rng） =====
fn alpha(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    match args.len() {
        0 => {
            let c = with_rng(|rng| rng.random::<char>());
            Ok(Expression::String(c.to_string()))
        }
        1 => {
            if let Expression::Integer(size) = &args[0] {
                let a = with_rng(|rng| rand::distr::Alphabetic.sample_string(rng, *size as usize));
                Ok(Expression::String(a))
            } else {
                Err(RuntimeError::common(
                    "rand.alpha expected integer size".into(),
                    ctx.clone(),
                    0,
                ))
            }
        }
        _ => Err(RuntimeError::common(
            "rand.alpha expected 0 or 1 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

fn alphanum(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    match args.len() {
        0 => {
            let c = with_rng(|rng| rng.sample(rand::distr::Alphanumeric)) as char;
            Ok(Expression::String(c.to_string()))
        }
        1 => {
            if let Expression::Integer(size) = &args[0] {
                let a =
                    with_rng(|rng| rand::distr::Alphanumeric.sample_string(rng, *size as usize));
                Ok(Expression::String(a))
            } else {
                Err(RuntimeError::common(
                    "rand.alphanum expected integer size".into(),
                    ctx.clone(),
                    0,
                ))
            }
        }
        _ => Err(RuntimeError::common(
            "rand.alphanum expected 0 or 1 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

// ===== 数值随机 =====
fn int(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    match args.len() {
        0 => {
            let n: i64 = with_rng(|rng| rng.random());
            Ok(Expression::Integer(n))
        }
        1 => {
            if let Expression::Integer(max) = &args[0] {
                let n = if *max < 0 {
                    with_rng(|rng| rng.random_range(*max..=0))
                } else {
                    with_rng(|rng| rng.random_range(0..=*max))
                };
                Ok(Expression::Integer(n))
            } else {
                Err(RuntimeError::common(
                    "rand.int expected integer max".into(),
                    ctx.clone(),
                    0,
                ))
            }
        }
        2 => match (&args[0], &args[1]) {
            (Expression::Integer(l), Expression::Integer(h)) => {
                let n = with_rng(|rng| rng.random_range(*l..*h));
                Ok(Expression::Integer(n))
            }
            (l, h) => Err(RuntimeError::common(
                format!("rand.int expected two integers, but got {l} and {h}").into(),
                ctx.clone(),
                0,
            )),
        },
        _ => Err(RuntimeError::common(
            "rand.int expected 0, 1 or 2 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

fn float(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    match args.len() {
        0 => {
            let f: f64 = with_rng(|rng| rng.random());
            Ok(Expression::Float(f))
        }
        2 => {
            let lo = get_float_ref(&args[0], ctx)?;
            let hi = get_float_ref(&args[1], ctx)?;
            if lo >= hi {
                return Err(RuntimeError::common(
                    "rand.float expected min < max".into(),
                    ctx.clone(),
                    0,
                ));
            }
            let f = with_rng(|rng| rng.random_range(lo..hi));
            Ok(Expression::Float(f))
        }
        _ => Err(RuntimeError::common(
            "rand.float expected 0 or 2 arguments".into(),
            ctx.clone(),
            0,
        )),
    }
}

// ===== 集合操作 =====
fn choose(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("choose", &args, 1, ctx)?;
    match &args[0] {
        Expression::List(list) => Ok(with_rng(|rng| match list.choose(rng) {
            Some(s) => s.clone(),
            None => Expression::None,
        })),
        otherwise => Err(RuntimeError::common(
            format!("rand.choose expected a list, but got {otherwise}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn shuffle(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("shuffle", &args, 1, ctx)?;
    match &args[0] {
        Expression::List(list) => {
            let mut s = list.as_ref().clone();
            with_rng(|rng| s.shuffle(rng));
            Ok(Expression::from(s))
        }
        otherwise => Err(RuntimeError::common(
            format!("rand.shuffle expected a list, but got {otherwise}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn sample(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sample", &args, 2, ctx)?;
    let n = get_integer_ref(&args[1], ctx)?;
    if n < 0 {
        return Err(RuntimeError::common(
            "rand.sample expected non-negative n".into(),
            ctx.clone(),
            0,
        ));
    }
    match &args[0] {
        Expression::List(list) => {
            let result = with_rng(|rng| {
                list.as_ref()
                    .sample(rng, n as usize)
                    .cloned()
                    .collect::<Vec<_>>()
            });
            Ok(Expression::from(result))
        }
        otherwise => Err(RuntimeError::common(
            format!("rand.sample expected a list, but got {otherwise}").into(),
            ctx.clone(),
            0,
        )),
    }
}
