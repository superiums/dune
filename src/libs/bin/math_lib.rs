use crate::libs::BuiltinInfo;
use crate::libs::bin::into_lib::string as to_string;
use crate::libs::helper::{check_args_len, check_exact_args_len, get_integer_ref};
use crate::libs::lazy_module::LazyModule;
use crate::{Environment, Expression, Int, RuntimeError, RuntimeErrorKind, reg_info, reg_lazy};
use std::collections::BTreeMap;
use std::ops::Rem;

pub fn handle_math(arg: &str, ctx: &Expression) -> Result<Expression, RuntimeError> {
    match arg {
        "E" => Ok(Expression::Float(std::f64::consts::E)),
        "PI" => Ok(Expression::Float(std::f64::consts::PI)),
        "PHI" => Ok(Expression::Float(1.618_033_988_749_895_f64)),
        _ => Err(RuntimeError::common(
            "unkown const in MATH".into(),
            ctx.clone(),
            0,
        )),
    }
}

/// 数学常量（无参数）
pub fn regist_const_math() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
         E => "Euler’s number (e)", ""
         PI => "Archimedes’ constant (π)", ""
         PHI => "The golden ratio (φ)", ""
    })
}

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        // 数学常量（无参数）
         // e, pi, phi,
         // 基础数学函数
         max, min, sum, average, abs, clamp,
         // 位运算
         bit_and, bit_or, bit_xor, bit_not, bit_shl, bit_shr,
         //逻辑运算
         gt, ge, lt, le, eq, ne,
         // 三角函数（单位：弧度）
         sin, cos, tan, asin, acos, atan,
         // 双曲函数
         sinh, cosh, tanh, asinh, acosh, atanh,
         // π倍三角函数
         sin_pi, cos_pi, tan_pi,
         // 指数与对数
         pow, exp, exp2, sqrt, cbrt, log, log2, log10, ln,
         // 舍入函数
         floor, ceil, round, trunc,
         // 其他函数
         is_odd, is_even, signum,
         hypot, gcd, lcm, rem,
         to_degrees, to_radians,
         // from into lib:
         to_string,
    })
}

pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        // 基础数学函数
        max => "max value", "<num1> <num2>... | <array>"
        min => "min value", "<num1> <num2>... | <array>"
        sum => "sum of numbers", "<num1> <num2>... | <array>"
        average => "average of numbers", "<num1> <num2>... | <array>"
        abs => "absolute value", "<number>"
        clamp => "clamp value into [min,max]", "<value> <min> <max>"

        // 位运算
        bit_and => "bitwise AND", "<int1> <int2>"
        bit_or => "bitwise OR", "<int1> <int2>"
        bit_xor => "bitwise XOR", "<int1> <int2>"
        bit_not => "bitwise NOT", "<integer>"
        bit_shl => "shift left, bits 0-63", "<integer> <bits>"
        bit_shr => "shift right, bits 0-63", "<integer> <bits>"

        // 逻辑运算
        gt => "a > b?", "<a> <b>"
        ge => "a >= b?", "<a> <b>"
        lt => "a < b?", "<a> <b>"
        le => "a <= b?", "<a> <b>"
        eq => "a == b?", "<a> <b>"
        ne => "a != b?", "<a> <b>"

        // 三角函数（单位：弧度）
        sin => "sine", "<radians>"
        cos => "cosine", "<radians>"
        tan => "tangent", "<radians>"
        asin => "inverse sine", "<value>"
        acos => "inverse cosine", "<value>"
        atan => "inverse tangent", "<value>"

        // 双曲函数
        sinh => "hyperbolic sine", "<value>"
        cosh => "hyperbolic cosine", "<value>"
        tanh => "hyperbolic tangent", "<value>"
        asinh => "inverse hyperbolic sine", "<value>"
        acosh => "inverse hyperbolic cosine", "<value>"
        atanh => "inverse hyperbolic tangent", "<value>"

        // π倍三角函数
        sin_pi => "sin(x*π)", "<x>"
        cos_pi => "cos(x*π)", "<x>"
        tan_pi => "tan(x*π)", "<x>"

        // 指数与对数
        pow => "base^exponent", "<base> <exponent>"
        exp => "e^x", "<x>"
        exp2 => "2^x", "<x>"
        sqrt => "square root", "<number>"
        cbrt => "cube root", "<number>"
        log => "log base b of x", "<base> <x>"
        log2 => "log base 2", "<number>"
        log10 => "log base 10", "<number>"
        ln => "natural log", "<number>"

        // 舍入函数
        floor => "round down", "<number>"
        ceil => "round up", "<number>"
        round => "round to nearest", "<number>"
        trunc => "truncate decimal", "<number>"

        // 其他函数
        to_string => "to string", "<number>"
        is_odd => "is odd?", "<integer>"
        is_even => "is even?", "<integer>"
        signum => "sign: -1, 0, or 1", "<number>"
        hypot => "sqrt(x^2+y^2)", "<x> <y>"
        gcd => "greatest common divisor", "<int1> <int2>"
        lcm => "least common multiple", "<int1> <int2>"
        rem => "euclidean remainder", "<a> <b>"
        to_degrees => "radians to degrees", "<radians>"
        to_radians => "degrees to radians", "<degrees>"
    })
}

// Helper function to evaluate arguments to f64
fn eval_to_f64(
    args: Vec<Expression>,
    _env: &mut Environment,
    func_name: &str,
    ctx: &Expression,
) -> Result<Vec<f64>, RuntimeError> {
    args.iter()
        .map(|arg| match arg {
            Expression::Integer(i) => Ok(*i as f64),
            Expression::Float(f) => Ok(*f),
            e => Err(RuntimeError::common(
                format!("invalid {func_name} argument {e}").into(),
                ctx.clone(),
                0,
            )),
        })
        .collect()
}

// Helper function to collect arguments (used by max/min)

pub fn get_float_arg(expr: &Expression, ctx: &Expression) -> Result<f64, RuntimeError> {
    match expr {
        Expression::Integer(i) => Ok(*i as f64),
        Expression::Float(i) => Ok(*i),
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
// Basic Math Functions
pub fn max(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let mut best: Option<Expression> = None;

    for num in args {
        let val = match &num {
            Expression::Integer(i) => *i as f64,
            Expression::Float(f) => *f,
            _ => {
                return Err(RuntimeError::common(
                    "max requires numeric arguments".into(),
                    ctx.clone(),
                    0,
                ));
            }
        };

        let replace = match &best {
            Some(Expression::Integer(bi)) => val > *bi as f64,
            Some(Expression::Float(bf)) => val > *bf,
            Some(_) => unreachable!(),
            None => true,
        };

        if replace {
            best = Some(num);
        }
    }

    Ok(best.unwrap_or(Expression::None))
}

pub fn min(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let mut best: Option<Expression> = None;

    for num in args {
        let val = match &num {
            Expression::Integer(i) => *i as f64,
            Expression::Float(f) => *f,
            _ => {
                return Err(RuntimeError::common(
                    "min requires numeric arguments".into(),
                    ctx.clone(),
                    0,
                ));
            }
        };

        let replace = match &best {
            Some(Expression::Integer(bi)) => val < *bi as f64,
            Some(Expression::Float(bf)) => val < *bf,
            Some(_) => unreachable!(),
            None => true,
        };

        if replace {
            best = Some(num);
        }
    }

    Ok(best.unwrap_or(Expression::None))
}
fn clamp(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("clamp", &args, 3, ctx)?;

    let value_f = get_float_arg(&args[0], ctx)?;
    let min_f = get_float_arg(&args[1], ctx)?;
    let max_f = get_float_arg(&args[2], ctx)?;

    let (min_f, max_f) = if min_f <= max_f {
        (min_f, max_f)
    } else {
        (max_f, min_f)
    };

    let result = if value_f < min_f {
        args[1].clone()
    } else if value_f > max_f {
        args[2].clone()
    } else {
        args[0].clone()
    };

    Ok(result)
}
// Bitwise Operations
fn bit_and(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bit_and", &args, 2, ctx)?;

    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;

    Ok(Expression::Integer(a & b))
}

fn bit_or(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bit_or", &args, 2, ctx)?;

    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;

    Ok(Expression::Integer(a | b))
}

fn bit_xor(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bit_xor", &args, 2, ctx)?;

    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;

    Ok(Expression::Integer(a ^ b))
}

fn bit_not(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bit_not", &args, 1, ctx)?;

    let a = get_integer_ref(&args[0], ctx)?;

    Ok(Expression::Integer(!a))
}

fn bit_shl(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bit_shl", &args, 2, ctx)?;

    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;
    if !(0..=63).contains(&b) {
        return Err(RuntimeError::common(
            format!("shift amount {} out of range (0-63)", a).into(),
            ctx.clone(),
            0,
        ));
    }
    Ok(Expression::Integer(a << b))
}

fn bit_shr(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("bit_shr", &args, 2, ctx)?;

    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;
    if !(0..=63).contains(&b) {
        return Err(RuntimeError::common(
            format!("shift amount {} out of range (0-63)", a).into(),
            ctx.clone(),
            0,
        ));
    }
    Ok(Expression::Integer(a >> b))
}
// Comparison Functions
fn gt(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("gt", &args, 2, ctx)?;
    let base = get_float_arg(&args[0], ctx)?;
    let other = get_float_arg(&args[1], ctx)?;
    Ok(Expression::Boolean(base.gt(&other)))
}

fn ge(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ge", &args, 2, ctx)?;
    let base = get_float_arg(&args[0], ctx)?;
    let other = get_float_arg(&args[1], ctx)?;
    Ok(Expression::Boolean(base.ge(&other)))
}

fn lt(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("lt", &args, 2, ctx)?;
    let base = get_float_arg(&args[0], ctx)?;
    let other = get_float_arg(&args[1], ctx)?;
    Ok(Expression::Boolean(base.lt(&other)))
}

fn le(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("le", &args, 2, ctx)?;
    let base = get_float_arg(&args[0], ctx)?;
    let other = get_float_arg(&args[1], ctx)?;
    Ok(Expression::Boolean(base.le(&other)))
}

fn eq(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("eq", &args, 2, ctx)?;
    let base = get_float_arg(&args[0], ctx)?;
    let other = get_float_arg(&args[1], ctx)?;
    Ok(Expression::Boolean(base.eq(&other)))
}

fn ne(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ne", &args, 2, ctx)?;
    let base = get_float_arg(&args[0], ctx)?;
    let other = get_float_arg(&args[1], ctx)?;
    Ok(Expression::Boolean(base.ne(&other)))
}
// Basic Math Operations
fn abs(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("abs", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(i) => Ok(i.abs().into()),
        Expression::Float(f) => Ok(f.abs().into()),
        e => Err(RuntimeError::common(
            format!("invalid abs argument {e:?}").into(),
            ctx.clone(),
            0,
        )),
    }
}

pub fn sum(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let mut int_sum = 0;
    let mut float_sum = 0.0;
    let mut has_float = false;

    match args.len() {
        0 => {
            return Err(RuntimeError::common(
                "sum requires some arguments".into(),
                ctx.clone(),
                0,
            ));
        }
        1 => match args.into_iter().next().unwrap() {
            Expression::List(list) => {
                if list.iter().any(|item| item.type_name() == "Float") {
                    float_sum = list.iter().fold(0 as f64, |acc, x| {
                        acc + get_float_arg(x, ctx).unwrap_or(0 as f64)
                    });
                    has_float = true
                } else {
                    int_sum = list
                        .iter()
                        .fold(0, |acc, x| acc + get_integer_ref(x, ctx).unwrap_or(0));
                }
            }
            Expression::BSet(list) => {
                if list.iter().any(|item| item.type_name() == "Float") {
                    float_sum = list.iter().fold(0 as f64, |acc, x| {
                        acc + get_float_arg(x, ctx).unwrap_or(0 as f64)
                    });
                    has_float = true
                } else {
                    int_sum = list
                        .iter()
                        .fold(0, |acc, x| acc + get_integer_ref(x, ctx).unwrap_or(0));
                }
            }
            _ => {
                return Err(RuntimeError::common(
                    "sum requires numeric arguments".into(),
                    ctx.clone(),
                    0,
                ));
            }
        },
        2.. => {
            for num in args {
                match num {
                    Expression::Integer(i) => {
                        if has_float {
                            float_sum += i as f64;
                        } else {
                            int_sum += i;
                        }
                    }
                    Expression::Float(f) => {
                        if !has_float {
                            float_sum = int_sum as f64;
                            has_float = true;
                        }
                        float_sum += f;
                    }
                    _ => {
                        return Err(RuntimeError::common(
                            "sum requires numeric arguments".into(),
                            ctx.clone(),
                            0,
                        ));
                    }
                }
            }
        }
    };

    if has_float {
        Ok(Expression::Float(float_sum))
    } else {
        Ok(Expression::Integer(int_sum))
    }
}

pub fn average(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("average", &args, 2.., ctx)?;

    let mut sum = 0.0;
    let mut count = 0;

    match args.len() {
        0 => {
            return Err(RuntimeError::common(
                "average requires some arguments".into(),
                ctx.clone(),
                0,
            ));
        }
        1 => match args.into_iter().next().unwrap() {
            Expression::List(list) => {
                sum = list.iter().fold(0 as f64, |acc, x| {
                    acc + get_float_arg(x, ctx).unwrap_or(0 as f64)
                });
                count = list.len();
            }
            Expression::BSet(list) => {
                sum = list.iter().fold(0 as f64, |acc, x| {
                    acc + get_float_arg(x, ctx).unwrap_or(0 as f64)
                });
                count = list.len();
            }
            _ => {
                return Err(RuntimeError::common(
                    "average requires numeric arguments".into(),
                    ctx.clone(),
                    0,
                ));
            }
        },
        2.. => {
            for num in args {
                match num {
                    Expression::Integer(i) => {
                        sum += i as f64;
                        count += 1;
                    }
                    Expression::Float(f) => {
                        sum += f;
                        count += 1;
                    }
                    _ => {
                        return Err(RuntimeError::common(
                            "average requires numeric arguments".into(),
                            ctx.clone(),
                            0,
                        ));
                    }
                }
            }
        }
    };

    Ok(Expression::Float(sum / count as f64))
}
// Rounding Functions
fn floor(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("floor", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(i) => Ok((*i).into()),
        Expression::Float(f) => Ok(f.floor().into()),
        e => Err(RuntimeError::common(
            format!("invalid floor argument {e:?}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn ceil(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ceil", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(i) => Ok((*i).into()),
        Expression::Float(f) => Ok(f.ceil().into()),
        e => Err(RuntimeError::common(
            format!("invalid ceil argument {e:?}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn round(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("round", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(i) => Ok((*i).into()),
        Expression::Float(f) => Ok(f.round().into()),
        e => Err(RuntimeError::common(
            format!("invalid round argument {e:?}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn trunc(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("trunc", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(i) => Ok((*i).into()),
        Expression::Float(f) => Ok(f.trunc().into()),
        e => Err(RuntimeError::common(
            format!("invalid trunc argument {e:?}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn is_odd(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_odd", &args, 1, ctx)?;
    Ok(match &args[0] {
        Expression::Integer(i) => (i % 2 != 0).into(),
        Expression::Float(f) => ((*f as Int) % 2 != 0).into(),
        e => {
            return Err(RuntimeError::common(
                format!("invalid isodd argument {e}").into(),
                ctx.clone(),
                0,
            ));
        }
    })
}
fn is_even(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_even", &args, 1, ctx)?;
    Ok(match &args[0] {
        Expression::Integer(i) => (i % 2 == 0).into(),
        Expression::Float(f) => ((*f as Int) % 2 == 0).into(),
        e => {
            return Err(RuntimeError::common(
                format!("invalid is_even argument {e}").into(),
                ctx.clone(),
                0,
            ));
        }
    })
}

fn signum(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("signum", &args, 1, ctx)?;
    match &args[0] {
        Expression::Integer(i) => Ok(Expression::Integer(i.signum())),
        Expression::Float(f) => Ok(Expression::Float(f.signum())),
        e => Err(RuntimeError::common(
            format!("invalid signum argument {e:?}").into(),
            ctx.clone(),
            0,
        )),
    }
}

fn hypot(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("hypot", &args, 2, ctx)?;
    let nums = eval_to_f64(args, env, "hypot", ctx)?;
    Ok(nums[0].hypot(nums[1]).into())
}

fn gcd_impl(a: Int, b: Int) -> Int {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn gcd(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("gcd", &args, 2, ctx)?;
    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;
    Ok(Expression::Integer(gcd_impl(a, b)))
}

fn lcm(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("lcm", &args, 2, ctx)?;
    let a = get_integer_ref(&args[0], ctx)?;
    let b = get_integer_ref(&args[1], ctx)?;
    if a == 0 || b == 0 {
        return Ok(Expression::Integer(0));
    }
    let g = gcd_impl(a, b);
    Ok(Expression::Integer((a / g * b).abs()))
}

fn rem(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rem", &args, 2, ctx)?;
    match (&args[0], &args[1]) {
        (Expression::Integer(a), Expression::Integer(b)) => {
            if *b == 0 {
                return Err(RuntimeError::common(
                    "rem division by zero".into(),
                    ctx.clone(),
                    0,
                ));
            }
            Ok(a.checked_rem(*b).map_or(Expression::None, Expression::from))
        }
        _ => {
            let a = get_float_arg(&args[0], ctx)?;
            let b = get_float_arg(&args[1], ctx)?;
            Ok(Expression::Float(a.rem(b)))
        }
    }
}

fn to_degrees(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_degrees", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "to_degrees", ctx)?[0];
    Ok(x.to_degrees().into())
}

fn to_radians(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("to_radians", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "to_radians", ctx)?[0];
    Ok(x.to_radians().into())
}

// Mathematical Functions
fn sqrt(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sqrt", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "sqrt", ctx)?[0];
    Ok(x.sqrt().into())
}

fn cbrt(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("cbrt", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "cbrt", ctx)?[0];
    Ok(x.cbrt().into())
}

fn exp(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("exp", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "exp", ctx)?[0];
    Ok(x.exp().into())
}

fn exp2(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("exp2", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "exp2", ctx)?[0];
    Ok(x.exp2().into())
}

fn log(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("log", &args, 2, ctx)?;
    let floats = eval_to_f64(args, env, "log", ctx)?;
    Ok(floats[1].log(floats[0]).into())
}

fn log2(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("log2", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "log2", ctx)?[0];
    Ok(x.log2().into())
}

fn log10(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("log10", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "log10", ctx)?[0];
    Ok(x.log10().into())
}

fn ln(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("ln", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "ln", ctx)?[0];
    Ok(x.ln().into())
}

fn pow(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("pow", &args, 2, ctx)?;
    let nums = eval_to_f64(args, env, "pow", ctx)?;
    Ok(nums[0].powf(nums[1]).into())
}
// Trigonometric Functions
fn sin(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sin", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "sin", ctx)?[0];
    Ok(x.sin().into())
}

fn cos(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("cos", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "cos", ctx)?[0];
    Ok(x.cos().into())
}

fn tan(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("tan", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "tan", ctx)?[0];
    Ok(x.tan().into())
}

fn asin(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("asin", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "asin", ctx)?[0];
    Ok(x.asin().into())
}

fn acos(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("acos", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "acos", ctx)?[0];
    Ok(x.acos().into())
}

fn atan(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("atan", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "atan", ctx)?[0];
    Ok(x.atan().into())
}
// Hyperbolic Functions
fn sinh(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sinh", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "sinh", ctx)?[0];
    Ok(x.sinh().into())
}

fn cosh(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("cosh", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "cosh", ctx)?[0];
    Ok(x.cosh().into())
}

fn tanh(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("tanh", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "tanh", ctx)?[0];
    Ok(x.tanh().into())
}

fn asinh(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("asinh", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "asinh", ctx)?[0];
    Ok(x.asinh().into())
}

fn acosh(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("acosh", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "acosh", ctx)?[0];
    Ok(x.acosh().into())
}

fn atanh(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("atanh", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "atanh", ctx)?[0];
    Ok(x.atanh().into())
}
// Pi Multiple Functions
fn sin_pi(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("sinpi", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "sinpi", ctx)?[0];
    Ok((x * std::f64::consts::PI).sin().into())
}

fn cos_pi(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("cospi", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "cospi", ctx)?[0];
    Ok((x * std::f64::consts::PI).cos().into())
}

fn tan_pi(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("tanpi", &args, 1, ctx)?;
    let x = eval_to_f64(args, env, "tanpi", ctx)?[0];
    Ok((x * std::f64::consts::PI).tan().into())
}
