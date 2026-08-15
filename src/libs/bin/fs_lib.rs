use crate::{
    Environment, Expression, Int, RuntimeError,
    expression::FileSize,
    libs::{
        BuiltinInfo,
        helper::{check_args_len, check_exact_args_len, get_string_arg, get_string_ref},
        lazy_module::LazyModule,
    },
    reg_info, reg_lazy,
    utils::expand_home,
};

#[cfg(unix)]
use crate::libs::helper::get_integer_ref;
use crate::utils::{self, get_current_path, join_current_path};
use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::{collections::BTreeMap, path::PathBuf};
// use super::fs_ls::list_directory_wrapper;
use super::fs_ls::ls;

pub fn regist_lazy() -> LazyModule {
    reg_lazy!({
        ls, glob, tree, abs, canon,
        // modify
        mkdir, rmdir, mv, cp, rm, touch,
        // permission & link
        chmod, chown, symlink, read_link,
        // check
        exists, is_dir, is_file, size,
        // read and write,
        head, tail, read, write, append,
        // assist
        base_name, stem, extension, dir_name, parent, join,

    })
}
pub fn regist_info() -> BTreeMap<&'static str, BuiltinInfo> {
    reg_info!({
        ls => "list dir contents", "[-l|s|a|h|t|L|c|u|g|m|p|d|?] [path]"
        glob => "match files by pattern", "<pattern>"
        tree => "dir tree as nested map", "[depth=3] [path]"
        abs => "absolute path", "<path>"
        canon => "canonical path, resolves symlinks", "<path>"

        // modify
        mkdir => "create dir, incl parents", "<path>"
        rmdir => "remove empty dir", "<path>"
        mv => "move path", "<source> <destination>"
        cp => "copy path, recursive for dirs", "<source> <destination>"
        rm => "remove path, recursive for dirs", "<path>"
        touch => "create empty file, or update mtime if exists", "<path>"

        // permission & link
        chmod => "set unix permission mode", "<path> <mode:octal>"
        chown => "set unix owner, -1 to keep", "<path> <uid> <gid>"
        symlink => "create symlink", "<source> <link_path>"
        read_link => "read symlink target", "<link_path>"

        // check
        exists => "path exists?", "<path>"
        is_dir => "is dir?", "<path>"
        is_file => "is file?", "<path>"
        size => "get file size", "<path>"

        // read/write
        head => "first n lines", "<file> [n=10]"
        tail => "last n lines", "<file> [n=10]"
        read => "read file, text or bytes", "<file>"
        write => "create/overwrite file", "[content] <file>"
        append => "append to file, creates if missing", "<content> <file>"

        // assist
        base_name => "file name with extension", "<path>"
        stem => "file name without extension", "<path>"
        extension => "file extension", "<path>"
        dir_name => "dir part before last '/'", "<path>"
        parent => "parent dir path", "<path>"
        join => "join path segments", "<segment>..."
    })
}

// Helper Functions
fn build_directory_tree(path: &Path, max_depth: Option<Int>) -> BTreeMap<String, Expression> {
    let mut tree = BTreeMap::new();

    tree.insert(
        ".".into(),
        Expression::String(path.to_string_lossy().to_string()),
    );
    if path.parent().is_some() {
        tree.insert(
            "..".into(),
            Expression::String(path.to_string_lossy().to_string()),
        );
    }

    if let Some(0) = max_depth {
        return tree;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let child_path = entry.path();
            if let Ok(name) = entry.file_name().into_string() {
                if child_path.is_dir() {
                    let new_depth = max_depth.map(|d| d - 1);
                    tree.insert(
                        name,
                        Expression::from(build_directory_tree(&child_path, new_depth)),
                    );
                } else {
                    tree.insert(name, Expression::String(path.to_string_lossy().to_string()));
                }
            }
        }
    }

    tree
}
// File operation implementations (unchanged from previous version)
fn move_path(src: &Path, dst: &Path, ctx: &Expression) -> Result<(), RuntimeError> {
    if src == dst {
        return Ok(());
    }
    if dst.exists() {
        return Err(RuntimeError::common(
            format!("Destination exists: {}", dst.display()).into(),
            ctx.clone(),
            0,
        ));
    }
    std::fs::rename(src, dst)
        .map_err(|e| RuntimeError::from_io_error(e, "move".into(), ctx.clone(), 0))
}

fn copy_path(src: &Path, dst: &Path, ctx: &Expression) -> Result<(), RuntimeError> {
    if src == dst {
        return Ok(());
    }
    if dst.exists() {
        return Err(RuntimeError::common(
            format!("Destination exists: {}", dst.display()).into(),
            ctx.clone(),
            0,
        ));
    }

    if src.is_dir() {
        std::fs::create_dir_all(dst)
            .map_err(|e| RuntimeError::from_io_error(e, "create dirs".into(), ctx.clone(), 0))?;
        for entry in std::fs::read_dir(src)
            .map_err(|e| RuntimeError::from_io_error(e, "read dir".into(), ctx.clone(), 0))?
        {
            let entry = entry
                .map_err(|e| RuntimeError::from_io_error(e, "read entry".into(), ctx.clone(), 0))?;
            let dst_path = dst.join(entry.file_name());
            copy_path(&entry.path(), &dst_path, ctx)?;
        }
    } else {
        std::fs::copy(src, dst)
            .map_err(|e| RuntimeError::from_io_error(e, "copy".into(), ctx.clone(), 0))?;
    }
    Ok(())
}

// Directory Tree Functions
fn tree(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("tree", &args, 1..=2, ctx)?;

    let mut cwd = get_current_path(env);
    let mut max_depth = Some(3);

    match &args[0] {
        Expression::Integer(n) => {
            max_depth = Some(*n);
            if let Some(Expression::String(path_expr)) = args.get(1) {
                cwd = cwd.join(path_expr);
            }
        }
        Expression::String(path) | Expression::Symbol(path) => {
            cwd = cwd.join(path);
            if let Some(Expression::Integer(depth)) = args.get(1) {
                max_depth = Some(*depth);
            }
        }
        _ => (),
    }

    Ok(Expression::from(build_directory_tree(&cwd, max_depth)))
}
// File Reading Functions
fn head(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("head", &args, 1..=2, ctx)?;

    if let Expression::String(p) = &args[0] {
        let path = utils::canon(p, env)?;
        let n = match args.len() {
            2 => match &args[1] {
                Expression::Integer(n) => *n,
                _ => {
                    return Err(RuntimeError::common(
                        "First argument must be an integer".into(),
                        ctx.clone(),
                        0,
                    ));
                }
            },
            1 => 10,
            _ => unreachable!(),
        };

        let result = read_file_portion(&path, n, true)?;
        return Ok(Expression::String(result));
    }
    Err(RuntimeError::common(
        "path arg must be a string".into(),
        ctx.clone(),
        0,
    ))
}

fn tail(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("tail", &args, 1..=2, ctx)?;
    if let Expression::String(p) = &args[0] {
        let path = utils::canon(p, env)?;
        let n = match args.len() {
            2 => match &args[1] {
                Expression::Integer(n) => *n,
                _ => {
                    return Err(RuntimeError::common(
                        "lines arg must be an integer".into(),
                        ctx.clone(),
                        0,
                    ));
                }
            },
            1 => 10,
            _ => unreachable!(),
        };

        let result = read_file_portion(&path, n, false)?;
        return Ok(Expression::String(result));
    }
    Err(RuntimeError::common(
        "path arg must be a string".into(),
        ctx.clone(),
        0,
    ))
}

fn read_file_portion(path: &Path, n: i64, from_start: bool) -> Result<String, RuntimeError> {
    let contents = std::fs::read_to_string(path).map_err(|e| {
        RuntimeError::from_io_error(e, "read file portion".into(), Expression::None, 0)
    })?;

    let mut lines: Vec<&str> = contents.lines().collect();
    if !from_start {
        lines.reverse();
    }

    let portion = lines
        .into_iter()
        .take(n.max(0) as usize)
        .collect::<Vec<&str>>()
        .join("\n");

    Ok(portion)
}
// Path Operations
fn canon(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("canon", &args, 1, ctx)?;
    if let Expression::String(p) = &args[0] {
        let canon_path = utils::canon(p, env)?;
        return Ok(Expression::String(canon_path.to_string_lossy().into()));
    }
    Err(RuntimeError::common(
        "path arg must be a string".into(),
        ctx.clone(),
        0,
    ))
}

fn abs(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("abs", &args, 1, ctx)?;
    if let Expression::String(p) = &args[0] {
        let canon_path = utils::abs_check(p, env)?;
        return Ok(Expression::String(canon_path.to_string_lossy().into()));
    }
    Err(RuntimeError::common(
        "path arg must be a string".into(),
        ctx.clone(),
        0,
    ))
}
// Directory Operations
fn mkdir(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("mkdir", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(p, env);
    std::fs::create_dir_all(&path).map_err(|e| {
        RuntimeError::from_io_error(e, "create directory".into(), args[0].clone(), 0)
    })?;
    Ok(Expression::None)
}

fn rmdir(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rmdir", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(p, env);
    std::fs::remove_dir(&path).map_err(|e| {
        RuntimeError::from_io_error(e, "remove directory".into(), args[0].clone(), 0)
    })?;
    Ok(Expression::None)
}
// File Operations
fn mv(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("mv", &args, 2, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let src = utils::abs(p, env);
    let dst_str = get_string_ref(&args[1], ctx)?;
    let dst = if is_a_dir(dst_str, env) {
        let mut dpath = join_current_path(dst_str, env);
        dpath.push(src.file_name().unwrap_or(OsStr::new("")));
        dpath
    } else {
        join_current_path(dst_str, env)
    };

    move_path(&src, &dst, ctx)?;
    Ok(Expression::None)
}

fn cp(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("cp", &args, 2, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let src = utils::abs(p, env);

    let dst_str = get_string_ref(&args[1], ctx)?;
    let dst = if is_a_dir(dst_str, env) {
        let mut dpath = join_current_path(dst_str, env);
        dpath.push(src.file_name().unwrap_or(OsStr::new("")));
        dpath
    } else {
        join_current_path(dst_str, env)
    };

    copy_path(&src, &dst, ctx)?;
    Ok(Expression::None)
}

fn rm(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("rm", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(p, env);
    remove_path(&path)?;
    Ok(Expression::None)
}

fn remove_path(path: &Path) -> Result<(), RuntimeError> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| {
            RuntimeError::from_io_error(e, "remove directory all".into(), Expression::None, 0)
        })?;
    } else {
        std::fs::remove_file(path).map_err(|e| {
            RuntimeError::from_io_error(e, "remove file".into(), Expression::None, 0)
        })?;
    }
    Ok(())
}
// Path Check Functions
fn exists(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("exists", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(p, env);
    Ok(Expression::Boolean(path.exists()))
}

fn is_dir(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_dir", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(p, env);
    Ok(Expression::Boolean(path.is_dir()))
}

fn is_file(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("is_file", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(p, env);
    Ok(Expression::Boolean(path.is_file()))
}

fn size(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("size", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(&p, env);
    let metadata = std::fs::metadata(&path)
        .map_err(|e| RuntimeError::from_io_error(e, "read metadata".into(), args[0].clone(), 0))?;
    Ok(Expression::FileSize(FileSize::from_bytes(metadata.len())))
}

// File Content Operations
fn read(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("read", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::canon(p, env)?;

    // First try to read as text
    if let Ok(contents) = std::fs::read_to_string(&path) {
        return Ok(Expression::String(contents));
    }

    // Fall back to reading as bytes
    let bytes = std::fs::read(&path)
        .map_err(|e| RuntimeError::from_io_error(e, "read file".into(), args[0].clone(), 0))?;
    Ok(Expression::Bytes(bytes))
}
// write content (String/Bytes/other) to a File, or truncate-create if `content` is None.
fn write(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("write", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let a0 = it.next().unwrap();

    // 一个参数 => a0 是 path，没有内容（相当于 touch）
    // 两个参数 => a0 是内容（管道传入），a1 是 path
    let (content, path_expr) = match it.next() {
        Some(path_expr) => (Some(a0), path_expr),
        None => (None, a0),
    };

    let p = get_string_ref(&path_expr, ctx)?;
    let path = utils::abs(p, env);

    match content {
        // 只有 path：如果文件不存在则创建空白文件；已存在则保持不变
        None => {
            if !path.exists() {
                std::fs::File::create(&path).map_err(|e| {
                    RuntimeError::from_io_error(e, "create file".into(), path_expr.clone(), 0)
                })?;
            }
        }
        // 有内容：写入（覆盖）
        Some(contents) => {
            match contents {
                Expression::Bytes(bytes) => std::fs::write(&path, bytes),
                Expression::String(ct) => std::fs::write(&path, ct),
                other => std::fs::write(&path, other.to_string()),
            }
            .map_err(|e| RuntimeError::from_io_error(e, "write file".into(), path_expr, 0))?;
        }
    }

    Ok(Expression::None)
}

// append content to a file, creating it if needed.
fn append(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_args_len("append", &args, 1..=2, ctx)?;
    let mut it = args.into_iter();
    let a0 = it.next().unwrap();

    // 一个参数 => a0 是 path，没有内容（相当于 touch，确保文件存在）
    // 两个参数 => a0 是内容（管道传入），a1 是 path
    let (content, path_expr) = match it.next() {
        Some(path_expr) => (Some(a0), path_expr),
        None => (None, a0),
    };

    let p = get_string_ref(&path_expr, ctx)?;
    let path = utils::abs(p, env);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|e| RuntimeError::from_io_error(e, "open file".into(), path_expr.clone(), 0))?;

    if let Some(contents) = content {
        match contents {
            Expression::Bytes(bytes) => file.write_all(&bytes),
            Expression::String(ct) => file.write_all(ct.as_bytes()),
            other => file.write_all(other.to_string().as_bytes()),
        }
        .map_err(|e| RuntimeError::from_io_error(e, "write file".into(), path_expr, 0))?;
    }

    Ok(Expression::None)
}

// Pattern Matching
fn glob(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("glob", &args, 1, ctx)?;

    let p = get_string_ref(&args[0], ctx)?;

    let cwd = get_current_path(env);
    let mut results = Vec::new();

    for entry in glob::glob(p).map_err(|e| {
        RuntimeError::common(
            format!("Invalid glob pattern: {p} - {e}").into(),
            ctx.clone(),
            0,
        )
    })? {
        let path = entry
            .map_err(|e| RuntimeError::common(format!("Glob error: {e}").into(), ctx.clone(), 0))?;
        let display_path = path
            .strip_prefix(&cwd)
            .unwrap_or(&path)
            .display()
            .to_string();
        results.push(Expression::String(display_path));
    }

    Ok(Expression::from(results))
}
// Path Extraction
// 完整文件名（含扩展名）
fn base_name(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("base_name", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let name = Path::new(p.as_str())
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Expression::String(name))
}

// 不含扩展名的主干名
fn stem(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("stem", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let name = Path::new(p.as_str())
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Expression::String(name))
}

// 单独获取扩展名
fn extension(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("extension", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let ext = Path::new(p.as_str())
        .extension()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(Expression::String(ext))
}

fn dir_name(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("dir_name", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;

    let dir = p
        .rfind("/")
        .map_or(String::from(""), |pos| p[..pos].to_string());

    Ok(Expression::String(dir))
}

fn parent(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("parent", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;

    let dir_name = match Path::new(&p).parent() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::from(""),
    };

    Ok(Expression::String(dir_name))
}

fn join(
    args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    // 检查至少有一个参数
    check_args_len("join", &args, 1.., ctx)?;

    let mut final_path = PathBuf::new();

    for arg in args {
        let p = get_string_arg(arg, ctx)?;
        final_path = final_path.join(p);
    }
    let p = expand_home(final_path.to_str().unwrap_or("."));
    // 返回合并后的路径作为 Expression
    Ok(Expression::String(p.into()))
}
fn is_a_dir(path: &str, env: &mut Environment) -> bool {
    let path = utils::abs(path, env);
    path.is_dir()
}

// touch: 文件不存在则创建空文件，存在则更新修改时间
fn touch(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("touch", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(&p, env);

    if path.exists() {
        let file = std::fs::File::open(&path)
            .map_err(|e| RuntimeError::from_io_error(e, "touch".into(), args[0].clone(), 0))?;
        file.set_modified(std::time::SystemTime::now())
            .map_err(|e| RuntimeError::from_io_error(e, "touch".into(), args[0].clone(), 0))?;
    } else {
        std::fs::File::create(&path)
            .map_err(|e| RuntimeError::from_io_error(e, "touch".into(), args[0].clone(), 0))?;
    }
    Ok(Expression::None)
}

// chmod: 仅 unix 支持，mode 为八进制整数（如 0o755）
#[cfg(unix)]
fn chmod(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    use std::os::unix::fs::PermissionsExt;

    check_exact_args_len("chmod", &args, 2, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(&p, env);
    let mode = get_integer_ref(&args[1], ctx)? as u32;

    let perms = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(&path, perms)
        .map_err(|e| RuntimeError::from_io_error(e, "chmod".into(), args[0].clone(), 0))?;
    Ok(Expression::None)
}

#[cfg(not(unix))]
fn chmod(
    _args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    Err(RuntimeError::common(
        "chmod is only supported on unix systems".into(),
        ctx.clone(),
        0,
    ))
}

// chown: 仅 unix 支持，uid/gid 传 -1 表示保持不变
#[cfg(unix)]
fn chown(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("chown", &args, 3, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(&p, env);
    let uid = get_integer_ref(&args[1], ctx)?;
    let gid = get_integer_ref(&args[2], ctx)?;

    let uid_opt = if uid < 0 { None } else { Some(uid as u32) };
    let gid_opt = if gid < 0 { None } else { Some(gid as u32) };

    std::os::unix::fs::chown(&path, uid_opt, gid_opt)
        .map_err(|e| RuntimeError::from_io_error(e, "chown".into(), args[0].clone(), 0))?;
    Ok(Expression::None)
}

#[cfg(not(unix))]
fn chown(
    _args: Vec<Expression>,
    _env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    Err(RuntimeError::common(
        "chown is only supported on unix systems".into(),
        ctx.clone(),
        0,
    ))
}

// symlink: 创建符号链接，source -> link_path
fn symlink(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("symlink", &args, 2, ctx)?;
    let src_s = get_string_ref(&args[0], ctx)?;
    let src = utils::abs(&src_s, env);
    let dst_s = get_string_ref(&args[1], ctx)?;
    let dst = join_current_path(&dst_s, env);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&src, &dst)
            .map_err(|e| RuntimeError::from_io_error(e, "symlink".into(), args[1].clone(), 0))?;
    }
    #[cfg(windows)]
    {
        let result = if src.is_dir() {
            std::os::windows::fs::symlink_dir(&src, &dst)
        } else {
            std::os::windows::fs::symlink_file(&src, &dst)
        };
        result.map_err(|e| RuntimeError::from_io_error(e, "symlink".into(), args[1].clone(), 0))?;
    }
    Ok(Expression::None)
}

// read_link: 读取符号链接指向的路径
fn read_link(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    check_exact_args_len("read_link", &args, 1, ctx)?;
    let p = get_string_ref(&args[0], ctx)?;
    let path = utils::abs(&p, env);

    let target = std::fs::read_link(&path)
        .map_err(|e| RuntimeError::from_io_error(e, "read_link".into(), args[0].clone(), 0))?;
    Ok(Expression::String(target.to_string_lossy().into()))
}
