#[cxx::bridge(namespace = "nix::rust_bindings")]
pub(crate) mod ffi {
    /// Nix logger verbosity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LogLevel {
        Error = 0,
        Warn = 1,
        Notice = 2,
        Info = 3,
        Talkative = 4,
        Chatty = 5,
        Debug = 6,
        Vomit = 7,
    }

    unsafe extern "C++" {
        include!("nix-bindings-logger/shim.hh");

        type BuildActivity;
        fn start_build_activity(description: &str) -> UniquePtr<BuildActivity>;
        fn build_log_line(activity: &BuildActivity, line: &str);
        fn log_message(level: LogLevel, message: &str);
    }
}
