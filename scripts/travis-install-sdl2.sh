#!/usr/bin/env bash
# From: https://github.com/Rust-SDL2/rust-sdl2/blob/master/scripts/travis-install-sdl2.sh

set -xueo pipefail

wget https://www.libsdl.org/release/SDL2-2.24.0.tar.gz -O sdl2.tar.gz
tar xzf sdl2.tar.gz
pushd SDL2-* && ./configure && make && sudo make install && popd
wget -q https://www.libsdl.org/projects/SDL_ttf/release/SDL2_ttf-2.20.1.tar.gz
wget -q https://www.libsdl.org/projects/SDL_image/release/SDL2_image-2.6.2.tar.gz
wget -q https://www.libsdl.org/projects/SDL_mixer/release/SDL2_mixer-2.6.2.tar.gz
wget -q -O SDL2_gfx-1.0.4.tar.gz https://sourceforge.net/projects/sdl2gfx/files/SDL2_gfx-1.0.4.tar.gz/download
tar xzf SDL2_ttf-*.tar.gz
tar xzf SDL2_image-*.tar.gz
tar xzf SDL2_mixer-*.tar.gz
tar xzf SDL2_gfx-*.tar.gz
pushd SDL2_ttf-* && ./configure && make && sudo make install && popd
pushd SDL2_image-* && ./configure && make && sudo make install && popd
pushd SDL2_mixer-* && ./configure && make && sudo make install && popd
pushd SDL2_gfx-* && ./autogen.sh && ./configure && make && sudo make install && popd
