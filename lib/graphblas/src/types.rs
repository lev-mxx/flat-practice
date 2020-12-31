#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::marker::PhantomData;
use std::os::raw::c_void;

use crate::*;

make_ffi_trait!(Type<T>);
make_ffi_static_struct!(Type<T>);

pub trait GrbType<T> {
    fn grb_type() -> &'static StaticType<T>;
}

#[macro_export]
macro_rules! for_each_type {
    ( $macro: ident ) => {
        _for_each_type!($macro,
            bool => BOOL,
            i8 => INT8,
            u32 => UINT32,
        );
    };
}

#[macro_export]
macro_rules! _for_each_type {
    ( $macro: ident, $($ty: ty => $grb_ty: ident),* $(,)? ) => {
        $($macro!($ty, $grb_ty);)*
    };
}

macro_rules! make_type {
    ( $ty:ty, $grb_ty:ident ) => {
        paste::paste! {
            make_static_instance!(Type<T> => <$ty>, [<GrB_ $grb_ty>], [<$ty _type>]);

            impl GrbType<$ty> for $ty {
                fn grb_type() -> &'static StaticType<$ty> { Type::<$ty>::[<$ty _type>]() }
            }
        }
    };
}

for_each_type!(make_type);
