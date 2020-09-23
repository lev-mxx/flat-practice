#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ptr;
use crate::*;

make_ffi_trait!(Descriptor);
make_ffi_static_struct!(Descriptor);

lazy_static! {
    static ref default_descriptor: StaticDescriptor = StaticDescriptor {
        link: GrbLink::of(ptr::null_mut()),
    };
}

impl dyn Descriptor {

    pub fn default() -> &'static StaticDescriptor { &default_descriptor }
}
