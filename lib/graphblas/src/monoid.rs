#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::os::raw::c_void;

use crate::*;

make_ffi_trait!(Monoid<T>);
make_ffi_static_struct!(Monoid<T>);

macro_rules! make_monoid {
    ($ty: ty,  $grb_ty: ty, $grb_op: ident, $op_name: ident) => {
        paste::paste! {
            make_static_instance!(Monoid<T> => <$ty>, [<GrB_ $grb_op _MONOID_ $grb_ty>], $op_name);
        }
    };
}

make_monoid!(bool, BOOL, LOR, lor);
make_monoid!(bool, BOOL, LAND, land);
