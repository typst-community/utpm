<div align="center">

![UTPM logo](./logo.svg)

> _Unofficial typst package manager_

**UTPM** is a _package manager_ for **[local](https://github.com/typst/packages#local-packages)** and **remote** packages. Create quickly new _projects_ and _templates_ from a **singular tool**, and then **publish** it _directly_ to **Typst**!

</div>

## 🔥 Features

- [x] ✨ Create packages automatically (`utpm create`)
  - [x] ⏯️ Interactive
  - [x] ⌨️ CLI version
- [x] 🛠 Put your package directly into your local packages (`utpm link`)
  - [ ] 💻 Link without copying! (`utpm link --no-copy`)
- [x] 🌐 Dependencies outsite typst!
  - [x] 📦 Install directly from the tool
  - [x] 🔒 Portable installer (limited for now)
- [x] 📃 List all your packages
  - [x] 🗃️ As a list `utpm list`
  - [x] 🌲 As a tree `utpm tree`
- [x] 💥 Customize your output (json or classic, `-j` in your commands)
- [x] 🗄️ Delete and bulk delete your packages (`utpm unlink`, `utpm bulk-delete`)
- [ ] 🚀 Publish it directly to Typst!

**_And many other features!_**

## 🔎 How to use it?

### The basic workflow

- _Firstly, you'll need to [create](#create) your `typst.toml` file!_
- _Then, edit your file! Like `index.typ` or `lib.typ`_
- _Finally, [link](#link) your new package to typst!_

### Commands

#### 🗄️ Bulk Delete

<!-- TODO: GIF -->

_A command to delete multiple packages at once!_

![bulk-delete.gif](./tapes/bulk_delete.gif)

<div id="create">

#### ✨ Create
_Create a `typst.toml` to make a package_

![create_cli.gif](./tapes/create_cli.gif)
<!-- TODO: GIF v2 -->

</div>
<div id="help">


#### ❓ Help

_Generate a help message_
![help.gif](./tapes/help.gif)

</div>
<div id="install">

#### 📦 Install
![install.gif](./tapes/install.gif)
<!-- TODO: GIF & text-->

</div>
<div id="link">

#### 🛠 Link
![link.gif](./tapes/link.gif)
<!-- TODO: GIF & text-->

</div>
<div id="list">

#### 🗃️ List
<!-- TODO: text -->

![list.gif](./tapes/list.gif)

</div>
<div id="package-path">
<!-- TODO: text -->

#### 🚦 Package Path

![packages-path.gif](./tapes/packages-path.gif)

</div>
<div id="tree">

#### 🌲 Tree

_A simple command to show all packages installed in your local dir like a tree!_

![tree.gif](./tapes/tree.gif)

</div>
<div id="unlink">
<!-- TODO: GIF -->

#### 🗄️ Unlink

![unlink.gif](./tapes/unlink.gif)

</div>

## ⚡ Install

You will need Cargo and Rust.

The easiest way to install utpm using Cargo is:

```bash
cargo install --git https://github.com/Thumuss/utpm
```

## Contribution

<!-- ndlr: on peut également dire "if you want to help me with developing this package" si ça sonne mieux pour toi -->
If you want to help me develop this package, simply make an issue or a PR!

By using this app, you contribute to it, thank you! <3
