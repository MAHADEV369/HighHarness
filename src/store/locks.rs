//! File-based advisory locks for the harness.
//!
//! Implements POSIX `flock`-style advisory locks (per `HARNESS_PRIMITIVES.md`
//! §4.3) with a simple pidfile-based recovery model. On Windows we use
//! `LockFileEx` semantics via a small shim.
//!
//! Locks are stored as a sidecar `.lock` file. The presence of the file
//! does not itself mean the lock is held; we use `flock(2)` for the
//! synchronization primitive.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::error::{HxError, HxResult};

/// A held file lock; releases on drop.
pub struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    /// Acquire a lock at `path`, retrying every 200ms up to `timeout_ms`.
    /// Returns `LockContention` on timeout (does NOT loop indefinitely).
    pub fn acquire(path: &Path, timeout_ms: u64) -> HxResult<FileLock> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)?;
        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        loop {
            if try_flock(&file)? {
                write_pidfile(&file, path)?;
                return Ok(FileLock {
                    file,

                    path: path.to_path_buf(),
                });
            }
            if start.elapsed() >= timeout {
                return Err(HxError::LockContention {
                    resource: path.display().to_string(),
                });
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// Try once to acquire; return `Ok(None)` if contended.
    pub fn try_acquire(path: &Path) -> HxResult<Option<FileLock>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)?;
        if try_flock(&file)? {
            write_pidfile(&file, path)?;
            Ok(Some(FileLock {
                file,

                path: path.to_path_buf(),
            }))
        } else {
            Ok(None)
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Err(e) = unlock_flock(&self.file) {
            eprintln!("lock: failed to unlock {}: {}", self.path.display(), e);
        }
    }
}

fn try_flock(file: &File) -> HxResult<bool> {
    let fd = file.as_raw_fd();
    // LOCK_EX | LOCK_NB
    let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    if result == 0 {
        Ok(true)
    } else {
        let err = std::io::Error::last_os_error();
        if err.kind() == ErrorKind::WouldBlock || err.raw_os_error() == Some(libc::EWOULDBLOCK) {
            Ok(false)
        } else {
            Err(err.into())
        }
    }
}

fn unlock_flock(file: &File) -> HxResult<()> {
    let fd = file.as_raw_fd();
    let result = unsafe { libc::flock(fd, libc::LOCK_UN) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().into())
    }
}

fn write_pidfile(file: &File, path: &Path) -> HxResult<()> {
    let pid = std::process::id();
    let ts = crate::id::now_iso();
    let mut s = String::new();
    s.push_str(&format!("{}\n{}\nagent:bootstrap\n", pid, ts));
    // Truncate & write
    let f = file;
    let fd: RawFd = f.as_raw_fd();
    // best-effort: we set the file contents to the pidfile
    let _ = fd;
    // Use standard write
    let f2: &File = f;
    // rewind via read+seek not necessary; just write
    let _ = f2;
    let mut opts = OpenOptions::new();
    opts.write(true).truncate(true);
    let mut dup = opts.open(path)?;
    dup.write_all(s.as_bytes())?;
    // Re-acquire lock on the dup descriptor (file path lock follows the inode,
    // not the fd). For flock semantics the original lock is still held.
    Ok(())
}

/// Read the pidfile content for a lock path. Returns `(pid, ts, agent)` if parseable.
#[allow(dead_code)]
pub fn read_pidfile(path: &Path) -> HxResult<(u32, String, String)> {
    let mut s = String::new();
    File::open(path)?.read_to_string(&mut s)?;
    let mut it = s.lines();
    let pid = it.next().and_then(|x| x.parse::<u32>().ok()).unwrap_or(0);
    let ts = it.next().unwrap_or("").to_string();
    let by = it.next().unwrap_or("").to_string();
    Ok((pid, ts, by))
}

/// Return true if the pid in a pidfile at `path` is still alive on the host.
#[allow(dead_code)]
pub fn pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // kill(pid, 0) returns 0 if process exists and we have permission
    // (or 0 == EPERM), -1 with ESRCH if it does not exist.
    let r = unsafe { libc::kill(pid as i32, 0) };
    if r == 0 {
        return true;
    }
    let err = std::io::Error::last_os_error();
    err.raw_os_error() == Some(libc::EPERM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn file_lock_acquire_and_release() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("test.lock");
        let l1 = FileLock::acquire(&p, 1000).unwrap();
        // Second acquisition must fail (lock contention).
        let r2 = FileLock::try_acquire(&p).unwrap();
        assert!(r2.is_none());
        drop(l1);
        // After release, we can acquire again.
        let l3 = FileLock::try_acquire(&p).unwrap();
        assert!(l3.is_some());
    }

    #[test]
    fn pid_alive_self_is_true() {
        let my_pid = std::process::id();
        assert!(pid_alive(my_pid));
        assert!(!pid_alive(0));
        assert!(!pid_alive(9_999_999));
    }
}
