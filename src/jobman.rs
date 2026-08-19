use std::process::Child;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum JobStatus {
    Running,
    Done(i32), // exit code
}

pub struct Job {
    pub id: u32,
    pub pid: u32,
    pub cmdline: String,
    pub child: Child,
    pub status: JobStatus,
}

static NEXT_JOB_ID: AtomicU32 = AtomicU32::new(1);
static JOBS: LazyLock<Mutex<Vec<Job>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// 注册一个新的后台任务，返回分配的 job id
pub fn add_job(child: Child, cmdline: String) -> u32 {
    let id = NEXT_JOB_ID.fetch_add(1, Ordering::SeqCst);
    let pid = child.id();
    if let Ok(mut jobs) = JOBS.lock() {
        jobs.push(Job {
            id,
            pid,
            cmdline,
            child,
            status: JobStatus::Running,
        });
    }
    // for ctrl+z job, need signal continue
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, libc::SIGCONT);
    }
    id
}

/// 惰性回收：对所有任务调用 try_wait()，更新已结束的状态
/// 返回值：仍在运行的任务数量
fn reap() -> usize {
    let mut running = 0;
    if let Ok(mut jobs) = JOBS.lock() {
        for job in jobs.iter_mut() {
            if job.status == JobStatus::Running {
                match job.child.try_wait() {
                    Ok(Some(status)) => {
                        job.status = JobStatus::Done(status.code().unwrap_or(-1));
                    }
                    Ok(None) => running += 1,
                    Err(_) => {
                        /* 保持 Running，下次再试 */
                        running += 1;
                    }
                }
            }
        }
    }
    running
}

/// 供 prompt / starship --jobs 使用：仅统计数量，不打印
pub fn running_count() -> usize {
    reap()
}

/// 供 `jobs` 内建命令使用：打印状态并做一次回收
pub fn list_jobs() -> Vec<(u32, u32, String, JobStatus)> {
    reap();
    if let Ok(mut jobs) = JOBS.lock() {
        let snapshot: Vec<_> = jobs
            .iter()
            .map(|j| (j.id, j.pid, j.cmdline.clone(), j.status))
            .collect();
        jobs.retain(|j| j.status == JobStatus::Running); // 展示后移除 Done 任务
        snapshot
    } else {
        Vec::new()
    }
}

/// kill 指定 job（用于 `kill %1` 之类的场景）
pub fn kill_job(job_id: u32) -> bool {
    if let Ok(mut jobs) = JOBS.lock()
        && let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            return job.child.kill().is_ok();
        }
    false
}
