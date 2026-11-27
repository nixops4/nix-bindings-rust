#include "nix-bindings-logger/shim.hh"

#include "nix-bindings-logger/src/bridge.rs.h"

namespace nix::rust_bindings {

static ::nix::Verbosity to_verbosity(LogLevel level) noexcept
{
    switch (level) {
    case LogLevel::Error:
        return ::nix::lvlError;
    case LogLevel::Warn:
        return ::nix::lvlWarn;
    case LogLevel::Notice:
        return ::nix::lvlNotice;
    case LogLevel::Info:
        return ::nix::lvlInfo;
    case LogLevel::Talkative:
        return ::nix::lvlTalkative;
    case LogLevel::Chatty:
        return ::nix::lvlChatty;
    case LogLevel::Debug:
        return ::nix::lvlDebug;
    case LogLevel::Vomit:
        return ::nix::lvlVomit;
    }

    return ::nix::lvlInfo;
}

std::unique_ptr<BuildActivity> start_build_activity(rust::Str description) noexcept
{
    try {
        if (!::nix::logger)
            return nullptr;
        return std::make_unique<BuildActivity>(std::string(description.data(), description.size()));
    } catch (...) {
        return nullptr;
    }
}

void build_log_line(const BuildActivity & activity, rust::Str line) noexcept
{
    try {
        activity.activity.result(
            ::nix::resBuildLogLine,
            std::string(line.data(), line.size()));
    } catch (...) {
    }
}

void log_message(LogLevel level, rust::Str message) noexcept
{
    try {
        if (!::nix::logger)
            return;
        ::nix::logger->log(
            to_verbosity(level),
            std::string_view(message.data(), message.size()));
    } catch (...) {
    }
}

} // namespace nix::rust_bindings
