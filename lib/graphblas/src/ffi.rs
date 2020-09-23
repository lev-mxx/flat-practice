#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::c_void;
use std::ffi::CStr;

pub struct GrbLink {
    link: *mut c_void,
}

pub(crate) trait GetLink {

    fn link(&self) -> *const c_void;
    fn link_mut(&mut self) -> *mut c_void;
}

impl GrbLink {

    pub(crate) fn of(link: *mut c_void) -> GrbLink { GrbLink { link } }

    pub(crate) fn link(&self) -> *const c_void { self.link }

    pub(crate) fn link_mut(&mut self) -> *mut c_void { self.link }
}

#[macro_export]
macro_rules! make_ffi_trait {
    ( $trait_name: ident $(<$( $i:ident ),*>)? ) => {
        paste::paste! {
            pub trait $trait_name$(<$($i),*>)? {
                fn grb_link(&self) -> &GrbLink;
                fn grb_link_mut(&mut self) -> &mut GrbLink;
            }
        }
    }
}

#[macro_export]
macro_rules! make_ffi_static_struct {
    ( $trait_name: ident $(<$( $i:ident ),*>)? ) => {
        paste::paste! {
            pub struct [<Static $trait_name>]$(<$($i),*>)? {
                link: GrbLink,
                $($([<_ $i>]: PhantomData<*const $i>),*)?
            }

            impl$(<$($i),*>)? $trait_name$(<$($i),*>)? for [<Static $trait_name>]$(<$($i),*>)? {
                fn grb_link(&self) -> &GrbLink { &self.link }

                fn grb_link_mut(&mut self) -> &mut GrbLink { &mut self.link }
            }

            unsafe impl$(<$($i),*>)? Sync for [<Static $trait_name>]$(<$($i),*>)? {}
        }
    };
}

#[macro_export]
macro_rules! make_static_instance {
    ($trait_name: ident $(<$( $i:ty ),*> => <$( $ty:ty ),*>)?, $grb_name: ident, $getter_name: ident) => {
        paste::paste! {
            #[link(name = "graphblas")]
            extern "C" {
                static $grb_name: *mut c_void;
            }

            lazy_static! {
                static ref [<instance_ $trait_name $($(_ $ty)*)? _ $getter_name>]: [<Static $trait_name>]$(<$($ty),*>)? = [<Static $trait_name>] {
                    link: GrbLink::of(unsafe {$grb_name}),
                    $($([<_ $i>]: PhantomData),*)?
                };
            }

            impl dyn $trait_name$(<$($ty),*>)? {
                pub fn $getter_name() -> &'static [<Static $trait_name>]$(<$($ty),*>)? { &[<instance_ $trait_name $($(_ $ty)*)? _ $getter_name>] }
            }
        }
    };
}

lazy_static! {
    pub(crate) static ref GRB: u32 = unsafe { GrB_init(GrB_Mode_GrB_NONBLOCKING) };
}

pub(crate) fn handle_grb_info(err: u32) {
    match err {
        0 => (),
        _ => {
            let grb_err_text = unsafe {
                CStr::from_ptr(GrB_error()).to_str()
            };

            panic!("Error: {}, grb error {:?} ", err, grb_err_text);
        }
    }
}

#[macro_export]
macro_rules! grb_call {
    ($fun: ident, $ty: ty, $($arg: expr),*) => {
        unsafe {
            let _ = *GRB;
            let mut M = MaybeUninit::<$ty>::uninit();
            handle_grb_info($fun(M.as_mut_ptr(), $($arg),*));
            M.assume_init()
        }
    };
}

#[macro_export]
macro_rules! grb_run {
    ($fun: ident, $($arg: expr),*) => {
        unsafe {
            let _ = *GRB;
            handle_grb_info($fun($($arg),*));
        }
    };
}

const GrB_Mode_GrB_NONBLOCKING: u32 = 0;

#[link(name = "graphblas")]
extern "C" {
    pub(crate) fn GrB_error() -> *const i8;
    fn GrB_init(mode: u32) -> u32;
}

