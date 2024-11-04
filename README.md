<div align="center">

![UTPM logo](./assets/logo.svg)

> _Unofficial typst package manager_

**UTPM** is a _package manager_ for **[local](https://github.com/typst/packages#local-packages)** and **[remote](https://github.com/typst/packages)** typst packages. Quickly create and manage _projects_ and _templates_ on your system, and publish them directly to **Typst Universe** via one unified tool.  

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

**_And more soon!_**

## 🔎 How to use it?

### The basic workflow

- _Firstly, you'll need to [create](#create) your `typst.toml` file!_
- _Then, edit your file! Like `index.typ` or `lib.typ`_
- _Finally, [link](#link) your new package to typst!_

### Commands

#### 🗄️ Bulk Delete

<!-- TODO: GIF -->

_A command to delete multiple packages at once!_

![bulk-delete.gif](./assets/gifs/bulk_delete.gif)

<div id="create">

#### ✨ Create
_Create a `typst.toml` to make a package_

![create_cli.gif](./assets/gifs/create_cli.gif)
<!-- TODO: GIF v2 -->

</div>
<div id="help">


#### ❓ Help

_Generate a help message_
![help.gif](./assets/gifs/help.gif)

</div>
<div id="install">

#### 📦 Install
![install.gif](./assets/gifs/install.gif)
<!-- TODO: GIF & text-->

</div>
<div id="link">

#### 🛠 Link
![link.gif](./assets/gifs/link.gif)
<!-- TODO: GIF & text-->

</div>
<div id="list">

#### 🗃️ List
<!-- TODO: text -->

![list.gif](./assets/gifs/list.gif)

</div>
<div id="package-path">
<!-- TODO: text -->

#### 🚦 Package Path

![packages-path.gif](./assets/gifs/packages-path.gif)

</div>
<div id="tree">

#### 🌲 Tree

_A simple command to show all packages installed in your local dir like a tree!_

![tree.gif](./assets/gifs/tree.gif)

</div>
<div id="unlink">
<!-- TODO: GIF -->

#### 🗄️ Unlink

![unlink.gif](./assets/gifs/unlink.gif)

</div>

## ⚡ Install

You will need Cargo and Rust.

The easiest way to install utpm using Cargo is:

```bash
$ cargo install --git https://github.com/Thumuss/utpm
```

## ❤️ Contribution

If you want to help me develop this package, simply make an issue or a PR!

By using this app, you contribute to it, thank you! <3
