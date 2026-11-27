#pragma once

#include <cstdint>
#include <memory>

#include "nix/util/logging.hh"
#include "rust/cxx.h"

namespace nix::rust_bindings {

enum class LogLevel : std::uint8_t;

// Opaque to Rust. Keeping Activity by value inside the heap allocation avoids
// moving Nix's non-movable activity after construction.
struct BuildActivity
{
    ::nix::Activity activity;

    explicit BuildActivity(const std::string & description)
        : activity(*::nix::logger, ::nix::lvlInfo, ::nix::actBuild, description)
    {
    }
};

std::unique_ptr<BuildActivity> start_build_activity(rust::Str description) noexcept;
void build_log_line(const BuildActivity & activity, rust::Str line) noexcept;
void log_message(LogLevel level, rust::Str message) noexcept;

} // namespace nix::rust_bindings
