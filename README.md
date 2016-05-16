# mazes

Fun with the [Rust](https://www.rust-lang.org/) programming language - maze generation, path finding and visualisation with [SDL](https://www.libsdl.org/).

[![Build status](https://api.travis-ci.org/enerqi/mazes.png)](https://travis-ci.org/enerqi/mazes)
[![Windows build status](https://ci.appveyor.com/api/projects/status/github/enerqi/mazes?svg=true)](https://ci.appveyor.com/project/enerqi/mazes)
[![Coverage Status](https://coveralls.io/repos/github/enerqi/mazes/badge.svg?branch=master)](https://coveralls.io/github/enerqi/mazes?branch=master)
[![](https://img.shields.io/badge/License-Apache2-green.svg)](https://github.com/enerqi/mazes/blob/master/LICENSE-APACHE)
[![](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/enerqi/mazes/blob/master/LICENSE-MIT)

![Wilson Maze](resources/wilson-maze.jpg)


## Build Requirements

- Install Rust.

On unix/posix family systems install the C libraries:
- libsdl2-dev
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
cargo run -- render recursive-backtracker image --grid-size=60 --mark-start-end --colour-distances --show-path
cargo run -- render wilson text image --text-out=="maze.text" --grid-size=40
```

## Doc Links

**Rust Core**: [Rust api docs](https://doc.rust-lang.org/std/). [Rust By Example](http://rustbyexample.com/).

**SDL2**: [Rust sdl2](https://angrylawyer.github.io/rust-sdl2/sdl2/) + [Repository](https://github.com/AngryLawyer/rust-sdl2), [Rust sdl2-image src](https://github.com/xsleonard/rust-sdl2_image/blob/master/src/sdl2_image/), [Rust sdl2-ttf].

**Other Rust Libs**: [bitset](https://contain-rs.github.io/bit-set/bit_set/index.html), [docopt](http://burntsushi.net/rustdoc/docopt/), [fnv](https://doc.servo.org/fnv/), [itertools](https://bluss.github.io/rust-itertools/doc/itertools/index.html), [num](https://rust-num.github.io/num/num/index.html), [petgraph](https://bluss.github.io/petulant-avenger-graphlibrary/doc/petgraph/index.html), [rand](https://doc.rust-lang.org/rand/rand/index.html), [smallvec](http://doc.servo.org/smallvec/index.html), [walkdir](http://burntsushi.net/rustdoc/walkdir/).
