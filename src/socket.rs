use std::io;
use std::path::Path;

#[cfg(unix)]
pub use std::os::unix::net::{UnixListener, UnixStream};

#[cfg(windows)]
pub use uds_windows::UnixListener;

/// Returns the platform-specific socket path
pub fn get_socket_path() -> String {
    if let Ok(path) = std::env::var("APPJS_SOCKET") {
        return path;
    }

    #[cfg(windows)]
    {
        // On Windows, a file path is fine, but it typically lives in the temp directory
        let mut path = std::env::temp_dir();
        path.push("appjs.sock");
        path.to_string_lossy().to_string()
    }
    #[cfg(not(windows))]
    {
        // On Unix, /tmp is standard
        "/tmp/appjs.sock".to_string()
    }
}

pub fn bind_socket<P: AsRef<Path>>(path: P) -> io::Result<UnixListener> {
    let path = path.as_ref();
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
    UnixListener::bind(path)
}
