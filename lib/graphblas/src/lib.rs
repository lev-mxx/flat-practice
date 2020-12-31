#[macro_use]
extern crate lazy_static;

pub use crate::binary_op::*;
pub use crate::descriptor::*;
pub(crate) use crate::ffi::*;
pub use crate::matrix::*;
pub use crate::monoid::*;
pub use crate::semiring::*;
pub use crate::types::*;
pub use crate::unary_op::*;

mod binary_op;
mod descriptor;
mod ffi;
mod matrix;
mod monoid;
mod semiring;
mod types;
mod unary_op;

#[test]
fn matrices_test() {
    let mut a = Matrix::<u32>::new(2, 2);
    a.build(
        &[0, 0, 1, 1],
        &[0, 1, 0, 1],
        &[1, 2, 3, 5],
        BinaryOp::<u32, u32, u32>::first(),
    );
    let mut b = Matrix::<u32>::new(2, 2);
    b.build(
        &[0, 0, 1, 1],
        &[0, 1, 0, 1],
        &[5, 3, 2, 1],
        BinaryOp::<u32, u32, u32>::first(),
    );

    let c = Matrix::<u32>::mxm(Semiring::<u32>::plus_times(), &a, &b);

    assert_eq!(c.get(0, 0), Some(9));
    assert_eq!(c.get(0, 1), Some(5));
    assert_eq!(c.get(1, 0), Some(25));
    assert_eq!(c.get(1, 1), Some(14));
}
