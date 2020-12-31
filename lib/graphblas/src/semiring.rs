#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::os::raw::c_void;

use crate::*;

make_ffi_trait!(Semiring<T>);
make_ffi_static_struct!(Semiring<T>);

macro_rules! make_semiring {
    ($ty: ty, $grb_ty: ty, $grb_op: ident, $op_name: ident) => {
        paste::item! {
            make_static_instance!(Semiring<T> => <$ty>, [<GrB_ $grb_op _SEMIRING_ $grb_ty>], $op_name);
        }
    }
}

make_semiring!(bool, BOOL, LOR_LAND, lor_land);
make_semiring!(u32, UINT32, PLUS_TIMES, plus_times);
