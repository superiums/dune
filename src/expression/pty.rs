use super::terminal::{TerminalOps, get_terminal_impl};
use crate::utils::get_current_path;
use crate::{Environment, RuntimeErrorKind, childman};
#[cfg(unix)]
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{self, Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// ---- 新增：集中的 PTY 交互行为配置 ----
#[cfg(unix)]
const PTY_CMDS: &[&str] = &[
    "lume", "bash", "sh", "fish", "top", "btop", "vi", "passwd", "ssh", "script", "expect",
    "telnet", "screen", "tmux", "ftp", "sftp",
];
#[cfg(windows)]
const PTY_CMDS: &[&str] = &[
    "lume",
    "fish",
    "ssh",
    "telnet",
    "screen",
    "tmux",
    "cmd.exe",
    "PowerShell",
    "Cygwin",
    "WinPTY",
    "ConPTY",
];

pub fn needs_pty(cmdstr: &str) -> bool {
    PTY_CMDS.contains(&cmdstr)
}

/// 描述某个命令在 PTY 模式下的交互行为，
/// 替代原来分散的 `is_vi`/`is_shell` 硬编码判断。
#[derive(Clone, Copy)]
pub struct PtyProfile {
    /// 输出转发是否使用低延迟逐块 read 循环（全屏/交互式程序，如 shell 本身、vi、top）。
    /// 关闭时使用简单的 `io::copy` 阻塞转发，性能更好但延迟稍高，适合非全屏程序。
    pub low_latency_output: bool,
    /// 进入"输入模式"时需要预先发送的按键序列（目前仅 vi 系需要先按 `i` 进入 insert mode）
    pub enter_insert: Option<&'static [u8]>,
    /// 写完初始输入后需要发送的收尾按键序列（vi 系需要 Esc 退出 insert mode）
    pub exit_insert: Option<&'static [u8]>,
}

const DEFAULT_PROFILE: PtyProfile = PtyProfile {
    low_latency_output: false,
    enter_insert: None,
    exit_insert: None,
};

/// 需要 vi 风格“先进插入模式再写入初始输入”的命令
const VI_LIKE: &[&str] = &["vi", "vim", "nvim"];

/// 需要低延迟输出转发的全屏/交互式命令（shell 自身 + vi 系 + 其它 TUI 程序）
/// 注：这里应与 cmd_excutor.rs 里判断是否需要 mode=16(PTY) 的命令列表保持同源，
/// 建议后续把两处列表合并为一个 pub 常量，避免重复维护（见下方 cmd_excutor.rs 修改说明）。
const LOW_LATENCY_SHELLS: &[&str] = &[
    "bash", "lume", "sh", "fish", "zsh", "ssh", "scp", "sftp", "top", "btop",
];

/// 根据命令名查询它的 PTY 交互配置。
/// 新增一个需要特殊处理的交互程序时，只需要在这里加一条，
/// 不需要改动 exec_in_pty 内部的任何分支逻辑。
fn pty_profile(cmdstr: &str) -> PtyProfile {
    if VI_LIKE.contains(&cmdstr) {
        return PtyProfile {
            low_latency_output: true,
            enter_insert: Some(b"i"),
            exit_insert: Some(&[27u8]), // ESC
        };
    }
    if LOW_LATENCY_SHELLS.contains(&cmdstr) {
        return PtyProfile {
            low_latency_output: true,
            enter_insert: None,
            exit_insert: None,
        };
    }
    DEFAULT_PROFILE
}

// 使用 RAII 守卫确保终端模式恢复
struct TerminalGuard {
    terminal: Box<dyn TerminalOps>,
    #[cfg(unix)]
    saved_sigint: Option<SigAction>,
}

impl TerminalGuard {
    fn new(terminal: Box<dyn TerminalOps>) -> Result<Self, RuntimeErrorKind> {
        #[cfg(unix)]
        let saved_sigint = unsafe {
            // Step 1: temporarily ignore SIGINT while we enter raw mode
            match signal::sigaction(
                signal::Signal::SIGINT,
                &SigAction::new(SigHandler::SigIgn, SaFlags::SA_RESTART, SigSet::empty()),
            ) {
                Ok(prev) => {
                    // Step 2: set SIGINT to SIG_DFL (for PTY child via ISIG)
                    let _ = signal::sigaction(
                        signal::Signal::SIGINT,
                        &SigAction::new(SigHandler::SigDfl, SaFlags::SA_RESTART, SigSet::empty()),
                    );
                    Some(prev)
                }
                Err(_) => None,
            }
        };
        terminal.enable_raw_mode()?;
        Ok(Self {
            terminal,
            #[cfg(unix)]
            saved_sigint,
        })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.terminal.disable_raw_mode();
        #[cfg(unix)]
        if let Some(ref prev) = self.saved_sigint {
            // Restore the SIGINT handler that was active before PTY execution
            unsafe {
                let _ = signal::sigaction(signal::Signal::SIGINT, prev);
            }
        }
    }
}

pub fn exec_in_pty(
    cmdstr: &String,
    args: Option<Vec<String>>,
    env: &mut Environment,
    input: Option<Vec<u8>>,
) -> Result<Option<Vec<u8>>, RuntimeErrorKind> {
    let terminal = get_terminal_impl();

    // 信号处理由 TerminalGuard 内部的 save/restore 管理，不再单独调用 setup_signal_handlers
    let (w, h) = terminal.get_terminal_size();

    // 输入处理线程
    let running = Arc::new(AtomicBool::new(false));
    let running_clone = Arc::clone(&running);
    let running_clone2 = Arc::clone(&running);
    // 设置 Ctrl+C 处理
    #[cfg(windows)]
    terminal.handle_ctrl_c(Arc::clone(&running))?;
    // Guard
    let _terminal_guard = TerminalGuard::new(terminal)?;

    // 替代原来的 is_vi / is_shell 两个散落布尔量
    let profile = pty_profile(cmdstr.as_str());

    // pty
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: h,
            cols: w,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| RuntimeErrorKind::CustomError(e.to_string().into()))?;

    let pair_master_fd = pair.master.as_raw_fd();

    // Unix 特定的终端设置
    #[cfg(unix)]
    {
        if let Some(master_fd) = pair_master_fd {
            unsafe {
                let mut termios = std::mem::zeroed();
                if libc::tcgetattr(master_fd, &mut termios) == 0 {
                    // 配置终端属性...
                    // 输入控制
                    // termios.c_cc[libc::VEOF] = 4; // Ctrl+D
                    // termios.c_cc[libc::VEOL] = libc::_POSIX_VDISABLE; // 无 EOL
                    // termios.c_cc[libc::VEOL2] = libc::_POSIX_VDISABLE;
                    // termios.c_cc[libc::VERASE] = 0x7f; // ASCII DEL (Backspace)
                    // termios.c_cc[libc::VWERASE] = 0x17; // Ctrl+W
                    // termios.c_cc[libc::VKILL] = 0x15; // Ctrl+U
                    // termios.c_cc[libc::VREPRINT] = 0x12; // Ctrl+R
                    // termios.c_cc[libc::VINTR] = 0x03; // Ctrl+C
                    // termios.c_cc[libc::VQUIT] = 0x1c; // Ctrl+\
                    // termios.c_cc[libc::VSUSP] = 0x1a; // Ctrl+Z
                    // termios.c_cc[libc::VSTART] = 0x11; // Ctrl+Q
                    // termios.c_cc[libc::VSTOP] = 0x13; // Ctrl+S
                    // termios.c_cc[libc::VLNEXT] = 0x16; // Ctrl+V
                    // termios.c_cc[libc::VDISCARD] = 0x0f; // Ctrl+O
                    // termios.c_cc[libc::VMIN] = 1;
                    // termios.c_cc[libc::VTIME] = 0;

                    termios.c_lflag |= libc::ECHO | libc::ICANON;
                    termios.c_lflag |= libc::ISIG;
                    // termios.c_lflag &= !libc::ISIG;
                    termios.c_oflag |= libc::OPOST;
                    // if is_vi {
                    //     termios.c_iflag &= !libc::IXON; // 禁用流控制
                    // }
                    libc::tcsetattr(master_fd, libc::TCSANOW, &termios);
                }
            }
        }
    }

    let mut cmd = CommandBuilder::new(cmdstr);

    let current_dir = get_current_path(env);
    cmd.cwd(current_dir);

    if let Some(ag) = args {
        cmd.args(ag);
    }

    for (k, v) in env.get_bindings_string() {
        cmd.env(k, v);
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| RuntimeErrorKind::CustomError(e.to_string().into()))?;

    // 注册 PTY 子进程 PID 到 childman（用于强制终止）
    if let Some(pid) = child.process_id() {
        childman::set_child(pid);
    }

    let mut master_reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| RuntimeErrorKind::CustomError(e.to_string().into()))?;
    let mut master_writer = pair
        .master
        .take_writer()
        .map_err(|e| RuntimeErrorKind::CustomError(e.to_string().into()))?;

    // 输出转发线程：低延迟逐块 vs 简单 io::copy，由 profile 决定
    let use_low_latency = profile.low_latency_output;
    let _output_thread = if use_low_latency {
        thread::spawn(move || {
            loop {
                if running_clone2.load(Ordering::SeqCst) {
                    break;
                }

                // 将读取的数据输出到标准输出
                let mut buffer = [0u8; 1024];
                match master_reader.read(&mut buffer) {
                    Ok(_) => io::stdout().write_all(&buffer).unwrap(),
                    Err(_) => break,
                }
                let _ = io::stdout().flush();
                thread::yield_now();
            }
        })
    } else {
        thread::spawn(move || {
            let _ = io::copy(&mut master_reader, &mut io::stdout());
        })
    };

    let enter_insert = profile.enter_insert;
    let exit_insert = profile.exit_insert;
    let input_thread = thread::spawn(move || {
        if let Some(last_input) = input {
            if let Some(seq) = enter_insert {
                #[cfg(unix)]
                unsafe {
                    // master_fd 需要在外部提前从 pair.master.as_raw_fd() 拿到
                    for _ in 0..100 {
                        // 最多等待约 1 秒
                        let mut termios: libc::termios = std::mem::zeroed();
                        if let Some(master_fd) = pair_master_fd
                            && libc::tcgetattr(master_fd, &mut termios) == 0
                            && termios.c_lflag & libc::ECHO == 0
                        {
                            break; // vi 已经切到自己的 raw/noecho 模式，安全注入
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                let _ = master_writer.write_all(seq);
            }
            if let Err(e) = master_writer.write_all(&last_input) {
                eprintln!("Failed to write to master: {e}");
            }
            if let Err(e) = master_writer.flush() {
                eprintln!("Failed to flush master: {e}");
            }
            if let Some(seq) = exit_insert {
                let _ = master_writer.write_all(b"\n");
                let _ = master_writer.write_all(seq);
                thread::sleep(Duration::from_millis(50));
                let _ = master_writer.flush();
            }
        }

        let mut input_buffer = [0u8];
        loop {
            if running_clone.load(Ordering::SeqCst) {
                break;
            }
            match io::stdin().read_exact(&mut input_buffer) {
                Ok(_) => {
                    let _ = master_writer.write_all(&input_buffer);
                }
                Err(_) => break,
            }
            let _ = master_writer.flush();
            thread::yield_now();
        }
    });

    child.wait()?;
    running.store(true, Ordering::SeqCst);
    let _ = input_thread.join();
    if use_low_latency {
        let _ = _output_thread.join();
    }

    Ok(None)
}
