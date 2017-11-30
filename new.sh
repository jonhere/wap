#!/bin/sh -e
if [ ! -f hello_world_release.html ]; then
    echo Only to be run from wap project directory
    exit 1
fi

if [ "x$1" = "x" ]; then
    echo Creates a new cargo project out of wap template
    echo Usage new.sh project_directory_path
    exit 1
fi

mkdir $1
mkdir $1/src
mkdir $1/.cargo
echo '[build]' > $1/.cargo/config
echo 'target = "wasm32-unknown-unknown"' >> $1/.cargo/config

cp hello_world_release.html $1/$(basename $1)_release.html
sed -i -e s/Hello\ World/$(basename $1)/ $1/$(basename $1)_release.html
sed -i -e s/hello_world.wasm/$(basename $1).wasm/ $1/$(basename $1)_release.html
sed -i -e s/src\\/wap.js/target\\/wasm32-unknown-unknown\\/release\\/wap.js/ $1/$(basename $1)_release.html

cp examples/hello_world.rs $1/src/main.rs

echo '[package]' >> $1/Cargo.toml
echo 'name = "'$(basename $1)'"' >> $1/Cargo.toml
echo 'version = "0.1.0"' >> $1/Cargo.toml
echo '' >> $1/Cargo.toml
echo '[dependencies]' >> $1/Cargo.toml
echo 'wap = { path = "'$(pwd)'" }' >> $1/Cargo.toml

git -C $1 init
