#!/bin/bash

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

install_dep() {
    cd "$SCRIPT_DIR"
    git clone https://github.com/CSSLab/maia-chess
    git clone -b release/0.28 --recurse-submodules https://github.com/LeelaChessZero/lc0.git
}

build_lc0() {
    cd "$SCRIPT_DIR/lc0/"
    exec "./build.sh"
}

install_dep
build_lc0