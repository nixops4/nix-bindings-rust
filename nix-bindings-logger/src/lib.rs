//! Emit messages and build activities through the logger of the hosting Nix
//! process.
//!
//! Nix's C API does not expose logger emission. This crate provides that
//! capability through a small CXX bridge to Nix's process-global logger and
//! `nix::Activity`. It is intended for code loaded into a running Nix process,
//! such as a store plugin.

mod bridge;

use cxx::UniquePtr;

use crate::bridge::ffi;

pub use crate::bridge::ffi::LogLevel;

/// Emit a message through Nix's process-global logger.
///
/// The hosting process decides whether the message is visible based on its
/// configured verbosity and log format. Logging is a no-op if the process has
/// no active logger.
pub fn log(level: LogLevel, message: &str) {
    ffi::log_message(level, message);
}

/// A live Nix build activity.
///
/// Lines emitted through this handle appear in `nix build -L`. Dropping the
/// handle ends the activity.
#[must_use = "dropping the handle immediately ends the Nix build activity"]
pub struct BuildLog(UniquePtr<ffi::BuildActivity>);

// SAFETY: the handle only submits results to Nix's process-global logger,
// which synchronizes activity updates internally. The C++ activity object is
// never moved after construction.
unsafe impl Send for BuildLog {}
unsafe impl Sync for BuildLog {}

impl BuildLog {
    /// Start a build activity with a human-readable description.
    pub fn start(description: &str) -> Self {
        Self(ffi::start_build_activity(description))
    }

    /// Whether the hosting process had a logger with which to create the
    /// activity.
    pub fn is_active(&self) -> bool {
        !self.0.is_null()
    }

    /// Emit one line of build output. Non-UTF-8 bytes are rendered lossily.
    ///
    /// This is a no-op if Nix could not create the activity.
    pub fn line(&self, line: &[u8]) {
        if let Some(activity) = self.0.as_ref() {
            ffi::build_log_line(activity, &String::from_utf8_lossy(line));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_message() {
        log(LogLevel::Info, "nix-bindings-logger message smoke test");
    }

    #[test]
    fn emits_build_log_line() {
        let log = BuildLog::start("nix-bindings-logger activity smoke test");
        assert!(log.is_active());
        log.line(b"build log line");
    }

    #[test]
    fn accepts_non_utf8_build_output() {
        let log = BuildLog::start("nix-bindings-logger non-UTF-8 smoke test");
        log.line(b"invalid byte: \xff");
    }
}
