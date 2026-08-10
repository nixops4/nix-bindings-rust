# nix-bindings-logger

Emit log messages and build activities through the logger of the hosting Nix
process.

Nix's C API does not expose logger emission. This crate provides that capability
through a small CXX bridge to Nix's process-global logger and `nix::Activity`.
It is intended for code loaded into a running Nix process, such as a store
plugin.
