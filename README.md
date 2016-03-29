# mazes

Fun with the ![Rust](https://www.rust-lang.org/) programming language - maze generation, path finding and visualisation with [SDL](https://www.libsdl.org/).

![Basic Maze](resources/binary-tree.jpg)
![Sidewinder Maze](resources/sidewinder.jpg)



## Try It!

Use the mazes driver executable to try out the mazes library. The commandline interface is built with ![docopt](http://docopt.org/).

```bash
cargo run -- --help

# Examples
cargo run -- render sidewinder image --grid-size=100
cargo run -- render binary image --image-out="maze.bmp" --grid-size=200
```

## Documentation Links

### Rust Core

[Rust api docs](https://doc.rust-lang.org/std/)

[Rust By Example](http://rustbyexample.com/)

### SDL2

[Rust sdl2 docs](https://angrylawyer.github.io/rust-sdl2/sdl2/)

[Rust sdl2 github](https://github.com/AngryLawyer/rust-sdl2)

[Rust sdl2 image src](https://github.com/xsleonard/rust-sdl2_image/blob/master/src/sdl2_image/)

[Rust sdl2 ttf src](https://github.com/andelf/rust-sdl2_ttf/tree/master/src/sdl2_ttf)

### Other Rust Libs

[docopt docs](http://burntsushi.net/rustdoc/docopt/)

[itertools docs](https://bluss.github.io/rust-itertools/doc/itertools/index.html)

[num docs](https://rust-num.github.io/num/num/index.html)

[petgraph docs](https://bluss.github.io/petulant-avenger-graphlibrary/doc/petgraph/index.html)

[smallvec docs](http://doc.servo.org/smallvec/index.html)


## License

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0 http://www.apache.org/licenses/LICENSE-2.0 or the MIT license http://opensource.org/licenses/MIT, at your option. This file may not be copied, modified, or distributed except according to those terms.
