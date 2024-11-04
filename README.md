<div align="center">

![UTPM logo](./assets/logo.svg)

> _Unofficial typst package manager_

**UTPM** is a _package manager_ for **[local](https://github.com/typst/packages#local-packages)** and **[remote](https://github.com/typst/packages)** typst packages. Quickly create and manage _projects_ and _templates_ on your system, and publish them directly to **Typst Universe**.  

[![Thumuss - utpm](https://img.shields.io/static/v1?label=Thumuss&message=utpm&color=blue&logo=github)](https://github.com/Thumuss/utpm "Go to GitHub repo")
[![stars - utpm](https://img.shields.io/github/stars/Thumuss/utpm?style=social)](https://github.com/Thumuss/utpm)
[![forks - utpm](https://img.shields.io/github/forks/Thumuss/utpm?style=social)](https://github.com/Thumuss/utpm)
<br/>
[![GitHub tag](https://img.shields.io/github/tag/Thumuss/utpm?include_prereleases=&sort=semver&color=blue)](https://github.com/Thumuss/utpm/releases/)
[![License](https://img.shields.io/badge/License-MIT-blue)](#license)
[![issues - utpm](https://img.shields.io/github/issues/Thumuss/utpm)](https://github.com/Thumuss/utpm/issues)


</div>

## 🔥 Features
- [x] ✨Create packages rapidly (`utpm workspace create`)
  - [x] ⏯️ Alias shorthand e.g. (`workspace = ws`)
  - [x] ⌨️ Intuitive Clap CLI
- [x] 🛠 Manage existing packages (`utpm ws link --no-copy`)
  - [x] 🔗Quick linking of remote and local packages (`utpm workspace link`)
  - [x] 🗄️ Delete and bulk delete your packages (`utpm pkg unlink`, `utpm pkg bulk-delete`)
- [x] 🌐 Dependencies outside of Typst!
  - [x] 📦 Support for third party application and plugins
  - [x] 🔒 Portable installer (limited for now)
- [x] 📃 Visualization 
  - [x] 🗃️ list `utpm pkg list`
  - [x] 🌲 tree `utpm pkg tree`
- [ ] 🚀 Automated publication directly to Typst Universe!

**_...And more soon!_**

<div id="install">

## ⚡Install
Requires Cargo and Rust. 

```bash
$ cargo install --git https://github.com/Thumuss/utpm
```
<div/>

<div id="usage">

## Usage 

Further usage information can be found by running `utpm --help` or `utpm <command> --help` on any of the sub commands. Documentation is still in progress, feel free to ask questions in the issues section.

### General

```
utpm [OPTIONS] <COMMAND>

Options

    -v, --verbose <VERBOSE>: Enable verbose output for debugging.
    -h, --help: Display help information.
    -V, --version: Display the utpm version.
```

### Commands

**Workspace** (ws): Manage Your Project Workspace
    - link (l): Link your project to existing directories.
    - create (c) (Deprecated) : Creates a typst.toml file. Use init instead.
    - install (i): Install dependencies listed in typst.toml.
    - add (a): Add and install new dependencies.
    - delete (d): Remove specific dependencies.
    - init: Initialize a new workspace for a Typst package.
    - publish (p) (WIP): Intended for publishing packages.
    - clone (WIP): Clone an existing workspace.

**Packages** (pkg): Manage Typst Packages
    - tree (t): Display all packages in a directory as a tree.
    - list (l): List all packages in a directory in a flat list.
    - path (p): Show the path to the Typst packages folder.
    - unlink (u): Remove a previously installed package.
    - bulk-delete (bd): Delete multiple packages at once.

**generate** (gen): Generate Shell Completions


## ❤️ Contribution

If you want to help me develop this package, simply make an issue or a PR!

By using this app, you contribute to it, thank you! <3
