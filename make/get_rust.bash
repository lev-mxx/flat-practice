RUST_VERSION=rust-1.46.0-x86_64-unknown-linux-gnu

(source make/set_paths.bash && echo $PATH && source make/has_rust.bash) || (
    (test -f build/rust.tar.gz ||
        curl --proto '=https' --tlsv1.2 -sSf https://static.rust-lang.org/dist/$RUST_VERSION.tar.gz --output build/rust.tar.gz) &&
    tar -xzf build/rust.tar.gz -C build &&
    cd build/$RUST_VERSION &&
    ./install.sh --prefix=$(pwd)/..
)
