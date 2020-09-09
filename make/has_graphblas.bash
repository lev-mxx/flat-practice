ldconfig -N -v $(sed 's/:/ /g' <<< $LD_LIBRARY_PATH) 2> /dev/null | grep libgraphblas.so > /dev/null
