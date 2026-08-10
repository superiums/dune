use crate::libs::pretty_printer;
use crate::{RuntimeErrorKind, set_print_direct};

use crate::utils::{expand_home, is_cfm_mode};
use crate::with_print_direct;
use crate::{Environment, Expression, MAX_RUNTIME_RECURSION, MAX_SYNTAX_RECURSION, SyntaxError};
use crate::{SyntaxErrorKind, parse_script};
use std::collections::HashSet;
use std::fs::{create_dir, read_to_string, write};
use std::io::{self, Write};
use std::path::PathBuf;
use std::rc::Rc;

pub fn run_file(pb: PathBuf, env: &mut Environment) {
    match read_to_string(pb.clone()) {
        Ok(prelude) => {
            env.define(
                "SCRIPT",
                Expression::String(pb.to_string_lossy().to_string()),
            );
            let code = parse_and_eval(&prelude, env);
            std::process::exit(code as i32);
        }
        Err(e) => {
            eprintln!(
                "\x1b[31m[IO ERROR]\x1b[0mFailed to read file '{}':\n  {e}",
                pb.display()
            );
            std::process::exit(-1);
        }
    }
}

pub fn parse_with_mode(input: &str) -> Result<Expression, SyntaxError> {
    let cfm = is_cfm_mode(input);
    let r = parse(input);
    if cfm {
        match r {
            Ok(Expression::Symbol(s)) => {
                // 验证符号不是数字或特殊字符
                if s.chars().any(|c| c.is_control() || c == '\0') {
                    return Err(SyntaxError {
                        source: format!("{input}").into(),
                        kind: SyntaxErrorKind::InvalidCmdSymbol(input.to_string()),
                    });
                }
                Ok(Expression::Command(
                    Rc::new(Expression::Symbol(s)),
                    Rc::new(vec![]),
                ))
            }
            #[cfg(unix)]
            Ok(Expression::String(s)) if !s.ends_with("/") && s.contains("/") => {
                if s.chars().any(|c| c.is_control() || c == '\0') {
                    return Err(SyntaxError {
                        source: format!("{input}").into(),
                        kind: SyntaxErrorKind::InvalidCmdSymbol(input.to_string()),
                    });
                }
                // 验证路径格式
                if s.contains("..") && !s.starts_with("../") {
                    return Err(SyntaxError {
                        source: format!("{input}").into(),
                        kind: SyntaxErrorKind::InvalidCmdSymbol(input.to_string()),
                    });
                }
                Ok(Expression::Command(
                    Rc::new(Expression::Symbol(s)),
                    Rc::new(vec![]),
                ))
            }
            #[cfg(windows)]
            Ok(Expression::String(s))
                if (s.contains(":\\")
                    || s.contains(".\\")
                    || s.contains(":/")
                    || s.contains("./")) =>
            {
                if s.chars().any(|c| c.is_control() || c == '\0') {
                    return Err(SyntaxError {
                        source: format!("{input}").into(),
                        kind: SyntaxErrorKind::InvalidCmdSymbol(input.to_string()),
                    });
                }
                // 验证 Windows 路径格式
                if (s.contains(":\\") || s.contains(":/")) && s.len() < 3 {
                    return Err(SyntaxError {
                        source: format!("{input}").into(),
                        kind: SyntaxErrorKind::InvalidCmdSymbol(input.to_string()),
                    });
                }
                Ok(Expression::Command(
                    Rc::new(Expression::Symbol(s)),
                    Rc::new(vec![]),
                ))
            }
            other => other,
        }
    } else {
        r
    }
}
pub fn parse(input: &str) -> Result<Expression, SyntaxError> {
    // dbg!(&input);
    match parse_script(input) {
        Ok(result) => Ok(result),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(SyntaxError {
            source: format!("{input}   ").into(),
            kind: e,
        }),
        Err(nom::Err::Incomplete(_)) => Err(SyntaxError {
            source: input.into(),
            kind: SyntaxErrorKind::InternalError("incomplted".to_string()),
        }),
    }
}

pub fn check(input: &str) -> bool {
    parse_script(input).is_ok()
}
pub fn knock_validate(env: &mut Environment) {
    if let Some(validator) = env.get("LUME_KNOCK_VALIDATOR") {
        match validator {
            Expression::Function(..) | Expression::Lambda(..) => {
                let r = validator.apply(vec![]).eval_cmd(env);
                if let Ok(Expression::Boolean(false)) = r {
                    std::process::exit(1);
                }
            }
            _ => {}
        };
    }
}
/// return whether parse success. no matter execute result is.
pub fn parse_and_eval(text: &str, env: &mut Environment) -> u8 {
    if text.is_empty() {
        return 0;
    };

    let parsed = parse_with_mode(text);

    match parsed {
        Ok(expr) => {
            let val = expr.eval_cmd(env);
            match val {
                Ok(Expression::None) => {}
                Ok(m)
                    if matches!(
                        m,
                        Expression::Map(_)
                            | Expression::HMap(_)
                            | Expression::List(_)
                            | Expression::Table(_)
                    ) =>
                {
                    let _ = pretty_printer(&m);
                }
                Ok(result) => {
                    if with_print_direct(|v| v) {
                        match result {
                            // skip function and lambda define PD
                            Expression::Function(..) => {}
                            Expression::Lambda(..) => {}
                            r => println!("\n  >> [{}] <<\n{}", r.type_name(), r),
                        };
                    }
                }
                Err(e) if let RuntimeErrorKind::Exited(code) = e.kind => {
                    if code == 0 {
                        println!("exited");
                    } else {
                        eprintln!("exited with code {code}");
                    }
                    return code; //e.code ?
                }
                Err(e) => {
                    let _ = io::stdout().flush();
                    eprintln!("\x1b[31m_____________\x1b[0m\n{e}");
                    return e.code();
                }
            }

            return 0;
        }

        Err(e) => {
            let _ = io::stdout().flush();
            eprintln!("\x1b[31m[PARSE ERROR]\x1b[0m\n{e}");
            return e.code();
        }
    }
}

pub fn init_config(env: &mut Environment) {
    let profile = match env.get("LUME_PROFILE") {
        Some(p) => PathBuf::from(p.to_string()),
        _ => match dirs::config_dir() {
            Some(config_dir) => {
                let config_path = config_dir.join("lumesh");
                if !config_path.exists()
                    && let Err(e) = create_dir(&config_path)
                {
                    eprintln!("Error while create prelude dir: {e}");
                }
                config_path.join("config.lm")
            }
            _ => PathBuf::from(".lume_config"),
        },
    };

    // If file doesn't exist
    if !profile.exists() {
        let prompt = format!(
            "Could not find profile file at: {}\nWould you like me to write a default one? (Y/n)\n>>> ",
            profile.display()
        );

        let response = read_user_input(prompt);

        const INTRO_PRELUDE: &str = if cfg!(target_os = "macos") {
            include_str!("config/config_mac.lm")
        } else if cfg!(windows) {
            include_str!("config/config_win.lm")
        } else {
            include_str!("config/config.lm")
        };

        if response.is_empty() || response.to_lowercase() == "y" {
            if let Err(e) = write(&profile, INTRO_PRELUDE) {
                eprintln!("Error while writing prelude: {e}");
            }
            env.define(
                "SCRIPT",
                Expression::String(profile.to_string_lossy().to_string()),
            );
        }

        if parse_and_eval(INTRO_PRELUDE, env) > 0 {
            eprintln!("Sorry, the config seems has some issue");
        }
    } else {
        match read_to_string(profile.clone()) {
            Ok(prelude) => {
                env.define(
                    "SCRIPT",
                    Expression::String(profile.to_string_lossy().to_string()),
                );
                let _ = parse_and_eval(&prelude, env);
            }
            Err(_) => {
                eprintln!("Failed to load config");
            }
        }
    }

    if let Some(pd) = env.get("LUME_PRINT_DIRECT") {
        set_print_direct(pd.is_truthy());
    }
    if let Some(Expression::Integer(run_rec)) = env.get("LUME_MAX_RUNTIME_RECURSION") {
        // MAX_RUNTIME_RECURSION = run_rec as usize;
        MAX_RUNTIME_RECURSION.with_borrow_mut(|v| *v = run_rec as usize)
    }
    if let Some(Expression::Integer(run_rec)) = env.get("LUME_MAX_SYNTAX_RECURSION") {
        // MAX_SYNTAX_RECURSION = run_rec as usize;
        MAX_SYNTAX_RECURSION.with_borrow_mut(|v| *v = run_rec as usize)
    }

    // clear env `SCRIPT`, to let modman recognize current dir.
    env.undefine("SCRIPT");

    // cmds
    init_cmds(env);
}

fn init_cmds(env: &mut Environment) {
    if !env.has("IFS") {
        env.define("IFS", Expression::None);
    }
    if !env.has("LUME_IFS_MODE") {
        env.define("IFS", Expression::Integer(60));
    }
    #[cfg(unix)]
    let sp = ":";
    #[cfg(windows)]
    let sp = ";";
    if let Some(Expression::String(pathes)) = env.get("PATH") {
        let np = pathes
            .split_terminator(sp)
            .filter(|p| !p.is_empty()) // 可选：过滤空字符串
            .map(|p| expand_home(p))
            .collect::<HashSet<_>>() // 使用 HashSet 去重
            .into_iter()
            .collect::<Vec<_>>()
            .join(sp);
        unsafe {
            std::env::set_var("PATH", np.clone());
        }
        env.define_in_root("PATH", Expression::String(np));
    } else {
        #[cfg(unix)]
        env.define_in_root(
            "PATH",
            Expression::String("/usr/local/bin:/usr/sbin:/usr/bin:".to_owned()),
        );
        #[cfg(windows)]
        env.define_in_root(
            "PATH",
            Expression::String("C:\\windows\\system32;C:\\windows\\;".to_owned()),
        );
    }
}

pub fn read_user_input(prompt: impl ToString) -> String {
    print!("{}", prompt.to_string());
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin()
        .read_line(&mut input)
        .map_err(|e| eprintln!("Read Failed: {e}"));
    input.trim().to_owned()
}

pub const IFS_CMD: u8 = 1 << 1; // cmd str_arg
pub const IFS_FOR: u8 = 1 << 2; // for i in str; str |> do
pub const IFS_STR: u8 = 1 << 3; // string.split
pub const IFS_CSV: u8 = 1 << 4; // parse.to_csv
pub const IFS_PCK: u8 = 1 << 5; // ui.pick
pub fn ifs_contains(mode: u8, env: &mut Environment) -> bool {
    if let Some(Expression::Integer(m)) = env.get("LUME_IFS_MODE")
        && m as u8 & mode != 0
    {
        return true;
    }
    false
}
