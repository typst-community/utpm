<div align="center">

![UTPM logo](./assets/logo.svg)

> _Unofficial Typst package manager_

**UTPM** is a _package manager_ for **[local](https://github.com/typst/packages#local-packages)** and **[remote](https://github.com/typst/packages)** Typst packages. Quickly create and manage _projects_ and _templates_ on your system, and publish them directly to **Typst Universe**.  

[![typst-community - utpm](https://img.shields.io/static/v1?label=typst-community&message=utpm&color=blue&logo=github)](https://github.com/typst-community/utpm "Go to GitHub repo")
[![stars - utpm](https://img.shields.io/github/stars/typst-community/utpm?style=social)](https://github.com/typst-community/utpm)
[![forks - utpm](https://img.shields.io/github/forks/typst-community/utpm?style=social)](https://github.com/typst-community/utpm)
<br/>
[![GitHub tag](https://img.shields.io/github/tag/typst-community/utpm?include_prereleases=&sort=semver&color=blue)](https://github.com/typst-community/utpm/releases/)
[![License](https://img.shields.io/badge/License-MIT-blue)](#license)
[![issues - utpm](https://img.shields.io/github/issues/typst-community/utpm)](https://github.com/typst-community/utpm/issues)

</div>


> [!WARNING]  
> **UTPM** is still in active development, and some features may not be fully implemented. \
> We are searching for contributors — anything you can offer will be greatly appreciated!


## 🔥 Features

- [x] ✨Create packages rapidly (`utpm workspace create`)
  - [x] ⏯️ Alias shorthand e.g. (`workspace = ws`)
  - [x] ⌨️ Intuitive Clap CLI
- [x] 🛠 Manage existing packages (`utpm ws link --no-copy`)
  - [x] 🔗 Link remote and local packages (`utpm workspace link`)
  - [x] 🗄️ Delete and bulk delete your packages (`utpm pkg unlink`, `utpm pkg bulk-delete`)
- [x] 🌐 Dependencies outside of Typst!
  - [x] 📦 Support for third party application and plugins
  - [x] 🔒 Portable installer (limited for now)
- [x] 📃 Visualization 
  - [x] 🗃️ list `utpm pkg list`
  - [x] 🌲 tree ~~`utpm pkg tree`~~ (use `list --tree` instead)
- [x] 📤 Flexible output formats
  - [x] 📝 Classic text and JSON output (fully supported)
  - [x] ⚠️ YAML, HJSON, and TOML output (experimental, under active development, and not included in official utpm builds — requires manual build for access)

- [ ] 🚀 Automated publication directly to Typst Universe!

**_...And more soon!_**


<div id="install">

## ⚡Install
### With cargo
Requires Cargo and Rust. 

```bash
$ cargo install --git https://github.com/typst-community/utpm
```

<details>
<summary>
  
### With nix

</summary>

#### Nix with flakes enabled:

Get utpm for a bash session without installing it:

```bash
$ nix shell github:typst-community/utpm
```

Or if you use NixOS or home-manager with a flake, install it permanently in your `flake.nix` or your modules:

```nix
{
  inputs.utpm.url = "github:typst-community/utpm";
  # ...

  outputs = { self, nixpkgs, ... }@inputs: {
    # change `yourhostname` or `yourusername` to your actual hostname or username
    nixosConfigurations.yourhostname = nixpkgs.lib.nixosSystem { #or homeConfigurations.yourusername
      system = "x86_64-linux";
      modules = [
        # ...
        {
          environment.systemPackages = [ inputs.utpm.packages.${system}.default ]; #or home.packages
        }
      ];
    };
  };
}
```

#### Nix without flakes:

Clone the repo and then nix-build into the utpm directory:

```bash
git clone https://github.com/typst-community/utpm.git
cd utpm
nix-build
./result/bin/utpm
```
Utpm will be at `./result/bin/utpm`

</details>
<div/>

<div id="usage">

## 🎰 Usage 
Further usage information can be found by running `utpm --help` or `utpm <command> --help` on any of the sub commands. Documentation is still in progress, feel free to ask questions in the issues section. Currently the github documentation is pretty much a mirror of the help command.

```
Usage: utpm [OPTIONS] <COMMAND>

Commands:
  workspace  Create, edit, delete your workspace for your package [aliases: ws]
  packages   use packages related to Typst [aliases: pkg]
  generate   Generate shell completions [aliases: gen]
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose <VERBOSE>  Gives you more information, permet debug
  -h, --help               Print help
  -V, --version            Print version
```

**Workspace** (ws): Manage Your Project Workspace
- `link (l)`: Link your project to existing directories.
- `create (c) (Deprecated)`: Creates a typst.toml file. Use init instead.
- `install (i)`: Install dependencies listed in typst.toml.
- `add (a)`: Add and install new dependencies.
- `delete (d)`: Remove specific dependencies.
- `init`: Initialize a new workspace for a Typst package.
- `publish (p) (WIP)`: Intended for publishing packages.
- `clone (WIP)`: Clone an existing workspace.

**Packages** (pkg): Manage Typst Packages
- `tree (t)`: Display all packages in a directory as a tree.
- `list (l)`: List all packages in a directory in a flat list.
- `path (p)`: Show the path to the Typst packages folder.
- `unlink (u)`: Remove a previously installed package.
- `bulk-delete (bd)`: Delete multiple packages at once.

**generate** (gen): Generate Shell Completions

<div/>

<div id="contribution">

## ❤️ Contribution

If you want to help me develop this package, simply make an issue or a PR!

By using this app, you contribute to it, thank you! <3

</div>
