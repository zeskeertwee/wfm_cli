wfm_cli
=======

[![license](https://img.shields.io/crates/l/wfm_rs)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/wfm_rs)](https://crates.io/crates/wfm_cli)
[![GitHub](https://img.shields.io/github/v/release/zeskeertwee/wfm_cli?label=Github)](https://github.com/zeskeertwee/wfm_cli)

A command-line tool that screenshots the relic reward screen, and select the best reward, based on warframe.market platinum prices.

## Installation
Download the latest release from GitHub.
```bash
$ wget -O wfm_cli https://github.com/zeskeertwee/wfm_cli/releases/download/v0.1.0/wfm_cli_linux
$ chmod +x wfm_cli
```
The first time you start up the program, it will ask you to sign into warframe.market.

## Usage
Run the program, and press ~ when you get to the relic reward screen, it's that simple!
```bash
$ ./wfm_cli
```

## Platform support
- Linux - Has been tested on Linux with X11 and GNOME, but it should also work on other desktop enviroments.
- MacOS - Hasn't been tested, probabbly works.
- Windows - Fails to compile due to screenshot library not supporting windows.