use crate::{
    Environment, Expression, RuntimeError, VERSION,
    libs::{BuiltinInfo, lazy_module::LazyModule},
    reg_info, reg_lazy,
};
use common_macros::hash_map;
use std::collections::BTreeMap;
use std::env::current_exe;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        version,
        bin,
        prelude,
        history,
        info
    })
}
pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        version => "version",""
        bin => "bin path",""
        prelude => "prelude path",""
        history => "history path",""
        info => "all info",""
    })
}

fn info(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let info = hash_map! {
        String::from("author") => Expression::String("Santo; Superiums; Adam McDaniel".to_string()),
        String::from("git") => Expression::String("https://codeberg.org/santo/lumesh".to_string()),
        String::from("homepage") => Expression::String("https://www.lumesh.cc.cd".to_string()),
        String::from("version") => Expression::String(VERSION.to_string()),
        String::from("bin") => bin(vec![], env, ctx)?,

        String::from("license") => Expression::String("MIT".to_string()),
        String::from("prelude") => prelude(args, env, ctx)?,
        String::from("history") => history(vec![], env, ctx)?
    };
    Ok(Expression::from(info))
}
fn version(
    _args: Vec<Expression>,
    _env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    Ok(Expression::String(VERSION.to_string()))
}
fn bin(
    _args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    current_exe()
        .map(|b| Expression::String(b.to_string_lossy().to_string()))
        .map_err(|e| RuntimeError::from_io_error(e, "read current executor".into(), ctx.clone(), 0))
}
fn prelude(
    _args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if let Some(profile) = env.get("LUME_PROFILE") {
        return Ok(profile);
    }
    Ok(if let Some(c) = dirs::config_dir() {
        let prelude_path = c.join("lumesh").join("config.lm");
        if prelude_path.exists() {
            Expression::String(prelude_path.to_str().unwrap().to_string())
        } else {
            Expression::String(prelude_path.to_str().unwrap().to_string() + " !")
        }
    } else {
        Expression::String("config.lm".to_string())
    })
}
fn history(
    _args: Vec<Expression>,
    env: &mut Environment,
    _ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    if let Some(hist) = env.get("LUME_HISTORY_FILE") {
        return Ok(hist);
    }
    Ok(if let Some(c) = dirs::cache_dir() {
        let hist = c.join("lumesh").join("history.log");
        if hist.exists() {
            Expression::String(hist.to_string_lossy().to_string())
        } else {
            Expression::String(format!("{} !", hist.to_string_lossy()))
        }
    } else {
        Expression::String("history.log".to_string())
    })
}
