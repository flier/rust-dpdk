#!/bin/sh

set -e

if [ -z "$RTE_SDK" ]; then
    echo "RTE_SDK - Points to the DPDK installation directory."
    exit 1
fi

if [ -z "$RTE_TARGET" ]; then
    echo "RTE_TARGET - Points to the DPDK target environment directory."
    exit 1
fi

bindgen -builtins src/rte.h -o src/raw.rs -I $RTE_SDK/$RTE_TARGET/include

attrs="\
#![allow(dead_code)]\n\
#![allow(non_camel_case_types)]\n\
#![allow(non_snake_case)]\n\
#![allow(non_upper_case_globals)]\n\
"

sed -i "1i$attrs" src/raw.rs
