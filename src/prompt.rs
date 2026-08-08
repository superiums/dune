use std::env;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use common_macros::hash_map;

use crate::{CFM_CONFIG, Environment, Expression, STRICT_ENABLED};

// 提示符状态缓存
#[derive(Clone)]
struct PromptCache {
    dir: PathBuf,
    content: String,
    updated: bool,
}

#[derive(Clone)]
struct PromptEngine {
    starship: bool,
    lazy: u8,
    custom_template: Option<String>,
    template_func: Option<Expression>,
    template_continuation: Option<String>,
    cache: Arc<Mutex<PromptCache>>,
}
pub trait PromptEngineCommon {
    fn get_prompt(&self, status: i32, duration: u128) -> String;
    fn get_prompt_continuation(&self) -> String;
    fn set_dir_cache(&self, dir: PathBuf);
}
// struct MyPrompt {}

// impl PromptEngineCommon for MyPrompt {
//     fn get_prompt(&self) -> String {
//         if let Ok(cwd) = env::current_dir() {
//             if let Some(cwd_str) = cwd.to_str() {
//                 return format!("\x1b[1;34m(lumesh)\x1b[0m{} \x1b[32m❯\x1b[0m ", cwd_str);
//             }
//         }
//         ">> ".into()
//     }
//     fn get_incomplete_prompt(&self) -> String {
//         "... ".into()
//     }
// }
impl PromptEngineCommon for PromptEngine {
    fn set_dir_cache(&self, dir: PathBuf) {
        if self.lazy > 0
            && let Ok(mut cache) = self.cache.lock()
        {
            cache.dir = dir;
            cache.updated = false;
        }
    }
    // 核心提示符生成方法
    fn get_prompt(&self, status: i32, duration: u128) -> String {
        // dbg!("getting prompt");

        // 2. 生成新提示符
        match self.starship {
            false => {
                // 1. 检查缓存有效性
                if self.lazy > 1
                    && let Ok(cache) = self.cache.lock()
                    && cache.updated
                {
                    return cache.content.clone();
                }

                // 2. 渲染
                let prompt = if let Some(func) = &self.template_func {
                    self.render_from_func(func, status, duration)
                } else if let Some(template) = &self.custom_template {
                    self.render_template(template, status, duration)
                } else {
                    self.default_prompt()
                };

                // 3. 更新缓存
                if self.lazy > 0
                    && let Ok(mut cache) = self.cache.lock()
                {
                    cache.content = prompt.clone();
                    cache.updated = true;
                }

                prompt
            }
            true => self
                .get_starship_prompt(status, duration)
                .unwrap_or_else(|| self.default_prompt()),
        }
    }
    fn get_prompt_continuation(&self) -> String {
        match self.starship {
            true => self.get_starship_continue().unwrap_or("... ".into()),
            _ => self.template_continuation.clone().unwrap_or("... ".into()),
        }
    }
}
impl PromptEngine {
    fn get_cwd(&self) -> Option<PathBuf> {
        if self.lazy > 0
            && let Ok(cache) = self.cache.lock()
        {
            Some(cache.dir.clone())
        } else {
            env::current_dir().ok()
        }
    }
    fn render_from_func(&self, func: &Expression, status: i32, duration: u128) -> String {
        let cwd = self.get_cwd();

        if let Some(cwd_pb) = cwd {
            let cfm = CFM_CONFIG.with_borrow(|cfm| cfm == &Some(true));
            let strict = STRICT_ENABLED.with_borrow(|s| s == &true);
            let jobs = crate::jobman::running_count();
            let ctx = Expression::from(hash_map! {
                String::from("cfm") => Expression::from(cfm),
                String::from("strict") => Expression::from(strict),
                String::from("status") => Expression::from(status as i64),
                String::from("duration") => Expression::from(duration as i64),
                String::from("jobs") => Expression::from(jobs as i64),
            });
            let r = func
                .apply(vec![
                    Expression::String(cwd_pb.to_string_lossy().to_string()),
                    ctx,
                ])
                .eval(&mut Environment::new());
            return match r {
                Ok(s) => s.to_string(),
                _ => self.default_prompt(),
            };
        }
        self.default_prompt()
    }
    fn render_template(&self, template: &str, status: i32, duration: u128) -> String {
        // 实现简单的占位符替换
        let jobs = crate::jobman::running_count();

        let mut result = template
            .replace("$STATUS", if status == 0 { "OK" } else { "FAIL" })
            .replace("$DURATION", &duration.to_string())
            .replace("$JOBS", &jobs.to_string())
            .replace(
                "$CFM_TAG",
                if CFM_CONFIG.with_borrow(|cfm| cfm.is_none()) {
                    "AUTO"
                } else if CFM_CONFIG.with_borrow(|cfm| cfm == &Some(true)) {
                    "CFM"
                } else {
                    "NM"
                },
            )
            .replace(
                "$STRICT_TAG",
                if STRICT_ENABLED.with_borrow(|cfm| cfm == &true) {
                    "S"
                } else {
                    "F"
                },
            );

        let cwd = self.get_cwd();

        if let Some(cwd_pb) = cwd {
            result = if result.contains("$CWD_SHORT") {
                result.replace("$CWD_SHORT", &get_short_path(cwd_pb.as_path()))
            } else {
                #[cfg(unix)]
                if cwd_pb.starts_with("/home/")
                    && let Some(home_dir) = dirs::home_dir()
                {
                    let cwd_new_str = cwd_pb
                        .to_string_lossy()
                        .replace(home_dir.to_string_lossy().as_ref(), "~");
                    return result.replace("$CWD", &cwd_new_str);
                }
                result.replace("$CWD", &cwd_pb.to_string_lossy())
            };
        }
        // 可以扩展更多占位符...
        result
    }

    fn get_starship_prompt(&self, status: i32, duration: u128) -> Option<String> {
        let (width, _) = crossterm::terminal::size().unwrap_or((80, 157));
        let dir = env::current_dir().ok()?;
        let jobs = crate::jobman::running_count();

        // 异步调用 starship prompt 子进程
        let output = Command::new("starship")
            .arg("prompt")
            .arg(format!("--terminal-width={width}"))
            .arg(format!("--status={status}"))
            .arg(format!("--cmd-duration={duration}"))
            .arg(format!("--jobs={jobs}"))
            .arg(format!("--logical-path={}", dir.display()))
            .envs(env::vars())
            .env("STARSHIP_SHELL", "lume")
            .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        String::from_utf8(output.stdout).ok()
    }

    fn get_starship_continue(&self) -> Option<String> {
        let output = Command::new("starship")
            .arg("--continuation")
            .envs(env::vars())
            .env("STARSHIP_SHELL", "lume")
            // .current_dir(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        String::from_utf8(output.stdout).ok()
    }

    fn default_prompt(&self) -> String {
        // 简约但有用的默认提示符
        if let Ok(cwd) = env::current_dir()
            && let Some(cwd_str) = cwd.to_str()
        {
            #[cfg(windows)]
            return format!("(lumesh){cwd_str} ❯ ");
            #[cfg(unix)]
            return format!("\x1b[1;34m(lumesh)\x1b[0m{cwd_str} \x1b[32m❯\x1b[0m ");
        }
        ">> ".into()
    }
}

fn get_short_path(path: &Path) -> String {
    // 将路径的组件收集到一个向量中
    let components = path.components().collect::<Vec<_>>();
    // dbg!(&components, components.len());

    // #[cfg(windows)]
    // let is_home = false;

    // #[cfg(unix)]
    let is_home = dirs::home_dir().is_some_and(|home_dir| path.starts_with(home_dir));
    // #[cfg(unix)]
    if is_home
        && let Some(home_dir) = dirs::home_dir()
        && components.len() < 6
    {
        return path
            .to_string_lossy()
            .to_string()
            .replace(home_dir.to_str().unwrap(), "~");
    }

    // 检查路径组件数量

    if !is_home && components.len() < 5 {
        return path.to_string_lossy().to_string();
    }

    let sep = if cfg!(windows) { "\\" } else { "/" };

    let first_two: Vec<String> = match is_home {
        true => vec![
            "~".to_owned() + sep,
            components
                .get(3)
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .to_string(),
        ],
        false => components
            .iter()
            .take(2)
            .map(|comp| comp.as_os_str().to_string_lossy().to_string())
            .collect(),
    };
    let last_two: Vec<String> = components
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(|comp| comp.as_os_str().to_string_lossy().to_string())
        .collect();

    // 生成短路径格式
    format!("{}...{}{}", first_two.join(""), sep, last_two.join(sep))
}

pub fn get_prompt_engine(settings: Option<Expression>) -> Box<dyn PromptEngineCommon> {
    let (starship, lazy, template, template_continuation) = match settings {
        Some(Expression::Map(sets)) => {
            let starship = sets.get("starship").is_some_and(|st| st.is_truthy());

            let lazy = sets.get("lazy").map_or(0, |st| {
                if let Expression::Integer(x) = st {
                    *x as u8
                } else {
                    0
                }
            });
            let template = sets.get("prompt_template").cloned();
            let template_continuation = sets.get("prompt_continuation").map(|tc| tc.to_string());
            (starship, lazy, template, template_continuation)
        }

        _ => (false, 0, None, None),
    };

    Box::new(PromptEngine {
        starship,
        lazy,
        template_continuation,
        template_func: template.clone().and_then(|f| match f {
            Expression::Lambda(..) => Some(f),
            Expression::Function(..) => Some(f),
            _ => None,
        }),
        custom_template: template.and_then(|t| match t {
            Expression::String(p) => Some(p.clone()),
            _ => None,
        }),
        cache: Arc::new(Mutex::new(PromptCache {
            dir: env::current_dir().unwrap_or_default(),
            content: "> ".to_string(),
            updated: false,
        })),
    })
}
