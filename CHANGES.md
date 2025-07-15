# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 25.0.0-dev.6 - 2025-07-15
### Added
- Implement an experimental `asimov module config` command (#16 by @SamuelSarle)
### Changed
- Disable Cargo/Ruby/Python source modules, for now (#13 by @SamuelSarle)
### Fixed
- Fix color output in `asimov module install` log messages (by @SamuelSarle)

## 25.0.0-dev.5 - 2025-07-02
### Added
- `asimov module link` and `asimov module browse` (#12 by @SamuelSarle)
### Changed
- Silence uninstall error for module not in registry (#10 by @SamuelSarle)
- Remove OpenSSL build dependency (@SamuelSarle)

## 25.0.0-dev.4 - 2025-06-27
### Added
- Implement the `asimov module resolve` command
- Enhance `asimov module install` to fetch a precompiled release (#4)
- Remove libexec binaries and the manifest on uninstall
### Changed
- Show installed binary modules in `asimov module list`
- Improve error messages
- Bump the MSRV to 1.85 (2024 edition)

## 25.0.0-dev.3 - 2025-04-23

## 25.0.0-dev.2 - 2025-04-22

## 25.0.0-dev.1 - 2025-04-21

## 25.0.0-dev.0 - 2025-04-04
