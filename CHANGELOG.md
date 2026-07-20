# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-19

### Added

- `cargo project-root-patch install` for injecting a workspace-local
  `project-root` proxy.
- One-time vendoring of the original `project-root` 0.2.2 source through Cargo's
  configured crates.io registry.
- Safe, repeatable workspace manifest patching.
- CI-based crates.io publication and trusted-publishing migration instructions.

[Unreleased]: https://github.com/milyin/project-root-patch/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/milyin/project-root-patch/releases/tag/v0.1.0
