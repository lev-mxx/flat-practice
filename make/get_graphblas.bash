(source make/set_paths.bash && source make/has_graphblas.bash) || (
    (test -d build/GraphBLAS || git clone -b stable --single-branch https://github.com/DrTimothyAldenDavis/GraphBLAS build/GraphBLAS) &&
    cd build/GraphBLAS &&
    git pull &&
    make JOBS=$(nproc --all) &&
    cd - &&
    cmake -DCMAKE_INSTALL_PREFIX=build -S build/GraphBLAS -B build/GraphBLAS/build &&
    cmake --install build/GraphBLAS/build
)
		