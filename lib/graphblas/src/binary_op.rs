#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::os::raw::c_void;

use crate::*;

make_ffi_trait!(BinaryOp<A, B, C>);
make_ffi_static_struct!(BinaryOp<A, B, C>);

macro_rules! make_binary_op {
    ($ty1: ty, $ty2: ty, $ty3: ty, $grb_ty: ty, $grb_op: ident, $op_name: ident) => {
        paste::paste! {
            make_static_instance!(BinaryOp<A, B, C> => <$ty1, $ty2, $ty3>, [<GrB_ $grb_op _ $grb_ty>], $op_name);
        }
    }
}

macro_rules! make_binary_ops {
    ( $ty:ty, $grb_ty:ident ) => {
        make_binary_op!($ty, $ty, $ty, $grb_ty, FIRST, first);
        make_binary_op!($ty, $ty, $ty, $grb_ty, SECOND, second);
        make_binary_op!($ty, $ty, $ty, $grb_ty, PLUS, plus);
        make_binary_op!($ty, $ty, $ty, $grb_ty, MINUS, minus);
        make_binary_op!($ty, $ty, $ty, $grb_ty, TIMES, times);
        make_binary_op!($ty, $ty, $ty, $grb_ty, DIV, div);
        make_binary_op!($ty, $ty, $ty, $grb_ty, MIN, min);
        make_binary_op!($ty, $ty, $ty, $grb_ty, MAX, max);
    };
}

for_each_type!(make_binary_ops);

make_static_instance!(BinaryOp<A, B, C> => <bool, bool, bool>, GrB_LOR, lor);
make_static_instance!(BinaryOp<A, B, C> => <bool, bool, bool>, GrB_LAND, land);
make_static_instance!(BinaryOp<A, B, C> => <bool, bool, bool>, GrB_LXOR, lxor);
make_static_instance!(BinaryOp<A, B, C> => <bool, bool, bool>, GrB_LXNOR, lxnor);
