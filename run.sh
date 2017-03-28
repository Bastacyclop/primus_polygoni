#!/bin/bash

execho() { echo "\$ $@" ; "$@" ; }

mode=`echo -e 'staging\ndynamic' | sort -R | tail -n 1`
cargo run --release --bin $mode -- 64 256
