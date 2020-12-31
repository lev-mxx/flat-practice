#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::os::raw::c_void;

use crate::*;

make_ffi_trait!(UnaryOp<A, B>);
make_ffi_static_struct!(UnaryOp<A, B>);

macro_rules! make_unary_op {
    ($ty1: ty, $ty2: ty, $grb_ty: ty, $grb_op: ident, $op_name: ident) => {
        paste::item! {
            make_static_instance!(UnaryOp<A, B> => <$ty1, $ty2>, [<GrB_ $grb_op _ $grb_ty>], $op_name);
        }
    }
}

macro_rules! unary_ops_gen {
    ( $rust_tpe:ty, $grb_tpy:ident ) => {
        paste::paste! {
            make_unary_op!($rust_tpe, $rust_tpe, $grb_tpy, IDENTITY, identity);
        }
    };
}

for_each_type!(unary_ops_gen);
