# ASIMOV Module Command-Line Interface (CLI)

[![License](https://img.shields.io/badge/license-Public%20Domain-blue.svg)](https://unlicense.org)
[![Compatibility](https://img.shields.io/badge/rust-1.85%2B-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
[![Package](https://img.shields.io/crates/v/asimov-module-cli)](https://crates.io/crates/asimov-module-cli)

🚧 _We are building in public. This is presently under heavy construction._

## ✨ Features

ASIMOV Module CLI is a tool for managing locally installed [ASIMOV Modules].

- Install, inspect, and uninstall [ASIMOV Modules].
- 100% free and unencumbered public domain software.

## 🛠️ Prerequisites

- [Rust](https://rust-lang.org) 1.85+ (2024 edition)

## ⬇️ Installation

The intended installation method is through Homebrew.

### Installation via Homebrew

Module CLI can be installed along [ASIMOV CLI](https://github.com/asimov-platform/asimov-cli) through Homebrew:

```bash
brew tap asimov-platform/tap
brew install asimov-cli
```

### Installation from Source Code

#### Installation via Cargo

```bash
cargo install asimov-module-cli --version 25.0.0-dev.5
```

## 👉 Examples

If you installed through Homebrew you're able to invoke the module as `asimov module`, otherwise call the `asimov-module` executable directly.

### Install a module

```bash
asimov module install http
```

### Inspect a module

You can print package pages that the module's manifest defines, including but not limited to source code repository and other documentation:

```console
$ asimov module link openai
https://rubygems.org/gems/asimov-openai-module
```

Or you can directly open in a web browser:

```bash
asimov module browse openai
```

### List modules

Lists available and installed modules:

```bash
asimov module list
```

### Find out which module(s) are able to handle a resource

```console
$ asimov module resolve https://asimov.sh/
http
```

### Uninstall a module

```bash
asimov module uninstall http
```

## 📚 Reference

TBD

## 👨‍💻 Development

```bash
git clone https://github.com/asimov-platform/asimov-module-cli.git
```

---

[![Share on X](https://img.shields.io/badge/share%20on-x-03A9F4?logo=x)](https://x.com/intent/post?url=https://github.com/asimov-platform/asimov-module-cli&text=ASIMOV%20Module%20Command-Line%20Interface%20%28CLI%29)
[![Share on Reddit](https://img.shields.io/badge/share%20on-reddit-red?logo=reddit)](https://reddit.com/submit?url=https://github.com/asimov-platform/asimov-module-cli&title=ASIMOV%20Module%20Command-Line%20Interface%20%28CLI%29)
[![Share on Hacker News](https://img.shields.io/badge/share%20on-hn-orange?logo=ycombinator)](https://news.ycombinator.com/submitlink?u=https://github.com/asimov-platform/asimov-module-cli&t=ASIMOV%20Module%20Command-Line%20Interface%20%28CLI%29)
[![Share on Facebook](https://img.shields.io/badge/share%20on-fb-1976D2?logo=facebook)](https://www.facebook.com/sharer/sharer.php?u=https://github.com/asimov-platform/asimov-module-cli)
[![Share on LinkedIn](https://img.shields.io/badge/share%20on-linkedin-3949AB?logo=linkedin)](https://www.linkedin.com/sharing/share-offsite/?url=https://github.com/asimov-platform/asimov-module-cli)

[ASIMOV Modules]: https://asimov.directory/modules
