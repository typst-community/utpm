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

- [x] ✨ Create and initialize packages rapidly (`utpm project init`)
  - [x] ⏯️ Alias shorthands (e.g., `project` -> `prj`, `packages` -> `pkg`)
  - [x] ⌨️ Intuitive Clap CLI
- [x] 🛠️ Manage project dependencies
  - [x] ➕ Add dependencies (`utpm prj add`)
  - [x] ➖ Remove dependencies (`utpm prj delete`)
  - [x] 🔄 Sync dependencies to the latest versions (`utpm prj sync`)
  - [x] 📦 Install dependencies from `typst.toml` (`utpm prj install`)
- [x] 📦 Manage local and remote packages
  - [x] 📥 Clone packages from the Typst Universe (`utpm prj clone`)
  - [x] 🔗 Link local packages for development (`utpm prj link`)
  - [x] 🗑️ Unlink packages (`utpm pkg unlink`)
  - [x] ⬆️ Bump package version (`utpm prj bump`)
- [x] 🔎 Discover and inspect packages
  - [x] 🗃️ List local packages (`utpm pkg list`)
  - [x] 🌲 Tree view for packages (`utpm pkg list --tree`)
  - [x] ℹ️ Get package information from the remote (`utpm pkg get`)
  - [x] ✅ Check for new package versions without updating (`utpm prj sync -c`)
- [x] 📤 Flexible output formats
  - [x] 📝 Classic text and JSON output (fully supported)
  - [x] ⚠️ YAML, HJSON, and TOML output (experimental, under active development, and not included in official utpm builds — requires manual build for access)
- [ ] 🚀 Automated publication directly to Typst Universe!

**_...And more soon!_**


<div id="install">

## ⚡Install
### With cargo
Requires Cargo and Rust. 

#### Main branch
```bash
$ cargo install --git https://github.com/typst-community/utpm
```

#### Latest version

> Best choice

```bash
$ cargo binstall utpm
```

Or

```bash
$ cargo install utpm
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
An unofficial typst package manager for your projects

Usage: utpm [OPTIONS] <COMMAND>

Commands:
  project  Subcommands for managing the project project [aliases: prj]
  packages   Subcommands for managing local packages [aliases: pkg]
  generate   Generate shell completion scripts [aliases: g]
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose <VERBOSE>              Enable verbose logging for debugging purposes
  -o, --output-format <OUTPUT_FORMAT>  The output format for command results [possible values: json, yaml, toml, text, hjson]
  -D, --dry-run                        If you don't want to write anything on your disk
  -h, --help                           Print help (see more with '--help')
  -V, --version                        Print version
```

### **Project (`prj`)**: Manage Your Project Project
| Command | Alias | Description |
| :--- | :---: | :--- |
| `link` | `l` | Link the current project to the local package directory. |
| `install` | `i` | Install a package from a github URL into your local directory |
| `init` | `n` | Create a new `typst.toml` manifest for a project. |
| `clone` | `c` | Clone a package from the typst universe or a local directory. |
| `bump` | `b` | Bump the version of your package. |
| `sync` | `s` | Synchronise all your dependencies to their last version. |
| `publish` | `p` | **(WIP)** Intended for publishing packages. |

### **Packages (`pkg`)**: Manage Typst Packages
| Command | Alias | Description |
| :--- | :---: | :--- |
| `list` | `l` | List all packages in your local storage. |
| `path` | `p` | Display the path to the typst packages folder. |
| `unlink` | `u` | Delete a package from your local storage. |
| `get` | `g` | Get specific/all package from the remote. |

### **generate (`gen`)**: Generate Shell Completions

<div/>

<div id="contribution">

## ❤️ Contribution

If you want to help me develop this package, simply make an issue or a PR!

By using this app, you contribute to it, thank you! <3

</div>
