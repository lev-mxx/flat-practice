mkdir -p build
if ! source make/has_rust.bash; then
    export HOME=$(pwd)/build/home
    export PATH=$(pwd)/build/bin:$PATH
fi
if ! source make/has_graphblas.bash; then
    export C_INCLUDE_PATH=$(pwd)/build/include:$C_INCLUDE_PATH
    export LIBRARY_PATH=$(pwd)/build/lib:$(pwd)/build/lib64:$LIBRARY_PATH
    export LD_LIBRARY_PATH=$(pwd)/build/lib:$(pwd)/build/lib64:$LD_LIBRARY_PATH
fi
