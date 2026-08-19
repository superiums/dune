#[cfg(unix)]
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDateTime};

use crate::expression::FileSize;
use crate::expression::table::TableData;
use crate::utils::abs;
use crate::{Environment, Expression, RuntimeError};

#[derive(Default)]
pub struct LsOptions {
    pub detailed: bool,
    pub show_hidden: bool,
    pub follow_links: bool,
    pub human_readable: bool,
    pub unix_time: bool,
    // pub size_in_kb: bool,
    pub show_create_time: bool,
    pub show_user: bool,
    pub show_group: bool,
    pub show_mode: bool,
    pub show_path: bool,
    pub list_dir_itself: bool,
    pub help: bool,
}

pub fn parse_ls_args(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<(Vec<PathBuf>, LsOptions), RuntimeError> {
    let mut options = LsOptions {
        detailed: true,
        ..Default::default()
    };

    let mut paths = Vec::new();
    for arg in args {
        if let Expression::Symbol(s) | Expression::String(s) = arg {
            match s.strip_prefix("-") {
                Some(opt) => {
                    for char in opt.chars() {
                        match char {
                            'l' => {}
                            's' => options.detailed = false,
                            'a' => options.show_hidden = true,
                            'h' => options.human_readable = true,
                            't' => options.unix_time = true,
                            'L' => options.follow_links = true,
                            'c' => options.show_create_time = true,
                            'u' => options.show_user = true,
                            'g' => options.show_group = true,
                            'm' => options.show_mode = true,
                            'p' => options.show_path = true,
                            'd' => options.list_dir_itself = true, // 新增：像 ls -d 一样列目录本身
                            '?' => options.help = true,
                            other => {
                                return Err(RuntimeError::common(
                                    format!("unkown option for fs.ls: `{}`", other).into(),
                                    ctx.clone(),
                                    0,
                                ));
                            }
                        }
                    }
                }
                None => paths.push(abs(&s, env)),
            }
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    Ok((paths, options))
}

pub fn ls(
    args: Vec<Expression>,
    env: &mut Environment,
    ctx: &Expression,
) -> Result<Expression, RuntimeError> {
    let (paths, options) = parse_ls_args(args, env, ctx)?;
    if options.help {
        println!(
            r"-l: detailed
-s: shorted
-a: show hidden
-h: human readable
-t: unix time
-L: follow links
-c: show create time
-u: show user
-g: show group
-m: show mode
-p: show path"
        );
        return Ok(Expression::None);
    }

    let mut headers = vec!["name".to_string()];
    if options.detailed {
        headers.extend_from_slice(&[
            "type".to_string(),
            "size".to_string(),
            "modified".to_string(),
        ]);
        #[cfg(unix)]
        if options.follow_links {
            headers.push("target".to_string());
        }
    }
    if options.show_create_time {
        headers.push("created".to_string());
    }
    #[cfg(unix)]
    {
        if options.show_user {
            headers.push("user".to_string());
        }
        if options.show_group {
            headers.push("group".to_string());
        }
        if options.detailed || options.show_mode {
            headers.push("mode".to_string());
        }
    }
    if options.show_path {
        headers.push("path".to_string());
    }

    let multi = paths.len() > 1;
    let mut rows = Vec::new();

    for full_path in &paths {
        let meta = match std::fs::symlink_metadata(full_path) {
            Ok(m) => m,
            Err(e) => {
                // 单个路径出错不应中断其余路径的列出（对齐 ls 的行为）
                eprintln!("ls: cannot access '{}': {}", full_path.display(), e);
                continue;
            }
        };

        let is_dir = meta.is_dir()
            || (options.follow_links
                && meta.file_type().is_symlink()
                && std::fs::metadata(full_path)
                    .map(|m| m.is_dir())
                    .unwrap_or(false));

        if is_dir && !options.list_dir_itself {
            for entry in std::fs::read_dir(full_path)
                .map_err(|e| RuntimeError::from_io_error(e, "read dir".into(), ctx.clone(), 0))?
            {
                let entry = entry.map_err(|e| {
                    RuntimeError::from_io_error(e, "read entry".into(), ctx.clone(), 0)
                })?;
                let file_name = entry.file_name();
                if !options.show_hidden && file_name.to_string_lossy().starts_with('.') {
                    continue;
                }
                rows.push(get_file_expression(&entry, &options, Some(full_path), ctx)?);
            }
        } else {
            // 文件参数，或 -d 模式下的目录本身：无论是否以 '.' 开头都要显示
            // （用户显式点名的文件不应被隐藏过滤掉）
            let _ = multi; // 若要按来源分组，可在此利用 multi 追加一个来源列
            rows.push(get_path_expression(full_path, &options, ctx)?);
        }
    }

    // 按名称排序，与主流 ls 默认行为一致
    rows.sort_by(|a, b| match (&a[0], &b[0]) {
        (Expression::String(x), Expression::String(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    });

    let table_data = TableData::new(headers, rows);
    Ok(Expression::Table(table_data))
}

fn build_row(
    name: String,
    full_path: &Path,
    metadata: std::fs::Metadata,
    options: &LsOptions,
    base_path: Option<&Path>,
    ctx: &Expression,
) -> Result<Vec<Expression>, RuntimeError> {
    let mut row = Vec::new();
    row.push(Expression::String(name.clone()));

    if options.detailed {
        let file_type = detect_file_type(&metadata);
        row.push(Expression::String(file_type.to_string()));

        let size_expr = if options.human_readable {
            Expression::FileSize(FileSize::from_bytes(metadata.len()))
        } else {
            Expression::Integer(metadata.len() as i64)
        };
        row.push(size_expr);

        let modified = metadata
            .modified()
            .map_err(|e| RuntimeError::from_io_error(e, "read mtime".into(), ctx.clone(), 0))?;
        let time_expr = if options.unix_time {
            Expression::Integer(system_time_to_unix_duration(modified, ctx)?.as_secs() as i64)
        } else {
            Expression::DateTime(system_time_to_naive_datetime(modified, ctx)?)
        };
        row.push(time_expr);

        #[cfg(unix)]
        if options.follow_links {
            if file_type == "symlink" {
                row.push(
                    std::fs::read_link(full_path)
                        .ok()
                        .map(|t| Expression::String(t.to_string_lossy().into_owned()))
                        .unwrap_or(Expression::None),
                );
            } else {
                row.push(Expression::None);
            }
        }
    }

    if options.show_create_time {
        // 修复：这里原来错误地又取了一次 modified()，"创建时间"应使用 created()
        let created = metadata
            .created()
            .map_err(|e| RuntimeError::from_io_error(e, "read ctime".into(), ctx.clone(), 0))?;
        let time_expr = if options.unix_time {
            Expression::Integer(system_time_to_unix_duration(created, ctx)?.as_secs() as i64)
        } else {
            Expression::DateTime(system_time_to_naive_datetime(created, ctx)?)
        };
        row.push(time_expr);
    }

    #[cfg(unix)]
    {
        if options.show_user {
            row.push(Expression::Integer(metadata.uid() as i64));
        }
        if options.show_group {
            row.push(Expression::Integer(metadata.gid() as i64));
        }
        if options.detailed || options.show_mode {
            row.push(Expression::Integer(
                (metadata.permissions().mode() & 0o777) as i64,
            ));
        }
    }

    if options.show_path {
        let path_str = base_path
            .map(|p| p.join(&name))
            .unwrap_or_else(|| full_path.to_path_buf());
        let path_final = dunce::canonicalize(&path_str).unwrap_or(full_path.to_path_buf());
        row.push(Expression::String(
            path_final.to_string_lossy().into_owned(),
        ));
    }

    Ok(row)
}

// 目录条目走这个薄包装
pub fn get_file_expression(
    entry: &std::fs::DirEntry,
    options: &LsOptions,
    base_path: Option<&Path>,
    ctx: &Expression,
) -> Result<Vec<Expression>, RuntimeError> {
    let p = entry.path();
    let metadata = if options.follow_links {
        entry
            .metadata()
            .map_err(|e| RuntimeError::from_io_error(e, "read file meta".into(), ctx.clone(), 0))?
    } else {
        p.symlink_metadata()
            .map_err(|e| RuntimeError::from_io_error(e, "read symlink".into(), ctx.clone(), 0))?
    };
    let name = entry.file_name().to_string_lossy().into_owned();
    build_row(name, &p, metadata, options, base_path, ctx)
}

// 单个文件/目录参数走这个
fn get_path_expression(
    full_path: &Path,
    options: &LsOptions,
    ctx: &Expression,
) -> Result<Vec<Expression>, RuntimeError> {
    let metadata = if options.follow_links {
        std::fs::metadata(full_path)
    } else {
        std::fs::symlink_metadata(full_path)
    }
    .map_err(|e| RuntimeError::from_io_error(e, "read meta".into(), ctx.clone(), 0))?;

    let name = full_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| full_path.to_string_lossy().into_owned());

    build_row(name, full_path, metadata, options, full_path.parent(), ctx)
}

#[cfg(unix)]
fn detect_file_type(metadata: &std::fs::Metadata) -> &'static str {
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        "directory"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_symlink() {
        "symlink"
    } else if file_type.is_socket() {
        "socket"
    } else if file_type.is_block_device() {
        "block_device"
    } else if file_type.is_char_device() {
        "char_device"
    } else if file_type.is_fifo() {
        "fifo"
    } else {
        "unknown"
    }
}

#[cfg(windows)]
fn detect_file_type(metadata: &std::fs::Metadata) -> &'static str {
    let file_type = metadata.file_type();

    if file_type.is_dir() {
        "directory"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_symlink() {
        "symlink"
    } else {
        "unknown"
    }
}

// 辅助函数：将 SystemTime 转换为 UNIX 时间戳的 Duration
fn system_time_to_unix_duration(
    st: SystemTime,
    ctx: &Expression,
) -> Result<std::time::Duration, RuntimeError> {
    st.duration_since(UNIX_EPOCH)
        .map_err(|_| RuntimeError::common("SystemTime before UNIX EPOCH".into(), ctx.clone(), 0))
}

// 辅助函数：将 SystemTime 转换为 NaiveDateTime
fn system_time_to_naive_datetime(
    st: SystemTime,
    ctx: &Expression,
) -> Result<NaiveDateTime, RuntimeError> {
    let duration = system_time_to_unix_duration(st, ctx)?;
    Ok(
        DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
            .unwrap_or_default()
            .naive_local(), // NaiveDateTime::from_timestamp_opt(duration.as_secs() as i64, duration.subsec_nanos())
                            // .unwrap_or_default(),
    ) // 提供默认值以防转换失败
}

// fn format_system_time(time: SystemTime) -> String {
//     let datetime: chrono::DateTime<chrono::Local> =
//         (UNIX_EPOCH + time.duration_since(UNIX_EPOCH).unwrap()).into();
//     datetime.format("%Y-%m-%d %H:%M:%S").to_string()
// }

// fn human_readable_size(size: u64) -> String {
//     const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
//     let mut size = size as f64;
//     let mut unit_idx = 0;

//     while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
//         size /= 1024.0;
//         unit_idx += 1;
//     }

//     if unit_idx == 0 {
//         format!("{}", size)
//     } else {
//         format!("{:.1}{}", size, UNITS[unit_idx])
//     }
// }
