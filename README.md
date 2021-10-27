<!--
SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>

SPDX-License-Identifier: CC0-1.0
-->

# **Proj**ect **Var**iables

[![License: GPL-3.0-or-later](
    https://img.shields.io/badge/License-GPL%203.0+-blue.svg)](
    https://www.gnu.org/licenses/gpl-3.0.html)
[![REUSE status](
    https://api.reuse.software/badge/github.com/hoijui/projvar)](
    https://api.reuse.software/info/github.com/hoijui/projvar)
<!--
[![crates.io](
    https://img.shields.io/crates/v/projvar.svg)](
    https://crates.io/crates/projvar)
-->
[![Docs](
    https://docs.rs/projvar/badge.svg)](
    https://docs.rs/projvar)
[![dependency status](
    https://deps.rs/repo/github/hoijui/projvar/status.svg)](
    https://deps.rs/repo/github/hoijui/projvar)
[![Build status](
    https://github.com/hoijui/projvar/workflows/build/badge.svg)](
    https://github.com/hoijui/projvar/actions)

This tool tries to extract a certain small set
of project related properties,
using various sources.

## Example scenario

In your CI job:

1. Check out your repo
2. Run this tool (`projvar`)
   * it ensures a a few properties are known, for example:
     * `PROJECT_NAME="My Project"`
     * `PROJECT_VERSION="my-proj-1.2.3-44-ge73gf28"`
     * `PROJECT_REPO_WEB_URL="https://github.com/user/my-proj"`
   * it stores them somehow, typically into a file,
     using [BASH `source`](
     https://opensource.com/article/20/6/bash-source-command) compatible syntax
3. Run some other tool that uses these environment variables.
   For example, you may include it in a QRCode,
   which you then print onto your project.

## How to compile

You need to install Rust(lang) and Cargo.

Then get the whole repo plus git submodules with:

```bash
git clone --recurse-submodules https://github.com/hoijui/projvar.git
cd projvar
```

Then you can run:

```bash
scripts/build
```

If all goeas well, the executable can be found at `target/release/projvar`.

## Get the tool

As for now, you have two choices:

1. [Compile it](#how-to-compile) yourself
1. Download a Linux x86\_64 staticially linked binary from
   [the releases page](https://github.com/hoijui/projvar/releases)

## Usage

```bash
$ projvar --help
TODO
```

