# mazes

Fun with the [Rust](https://www.rust-lang.org/) programming language - maze generation, path finding and visualisation with [SDL](https://www.libsdl.org/).

[![Build status](https://api.travis-ci.org/enerqi/mazes.png)](https://travis-ci.org/enerqi/mazes)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/enerqi/mazes?svg=true)](https://ci.appveyor.com/project/enerqi/mazes)
[![codecov.io](http://codecov.io/github/enerqi/mazes/coverage.svg?branch=master)](http://codecov.io/gh/enerqi/mazes?branch=master)
[![](https://img.shields.io/badge/License-Apache2-green.svg)](https://github.com/enerqi/mazes/blob/master/LICENSE-APACHE)
[![](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/enerqi/mazes/blob/master/LICENSE-MIT)

![Wilson Maze](https://sublime.is/demo/mazes/wilson-maze.png)


## Build Requirements

- Install Rust 1.18+.

On unix/posix family systems install the C libraries:
- libsdl2-dev  >= 2.0.5
- libsdl2-ttf-dev
- libsdl2-image-dev

On windows and unix/posix install the tool:
- gcc

GCC is probably already installed on posix! For windows see e.g. [mingw-w64](http://mingw-w64.org/doku.php) or [mingw-w64 chocolatey](https://chocolatey.org/packages/mingw).

## Run It!

Use the mazes driver executable to try out the mazes library. The commandline interface is built with [docopt](http://docopt.org/).

```bash
cargo run -- --help

# Examples
cargo run -- render recursive-backtracker image --grid-width=140 --grid-height=80 --mark-start-end --colour-distances --show-path
cargo run -- render wilson text image --text-out="maze.text" --grid-size=40
```
