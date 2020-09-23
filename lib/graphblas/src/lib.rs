mod ffi;
mod matrix;
mod binary_op;
mod types;
mod descriptor;
mod monoid;
mod semiring;
mod unary_op;

pub(crate) use crate::ffi::*;
pub use crate::matrix::*;
pub use crate::types::*;
pub use crate::descriptor::*;
pub use crate::monoid::*;
pub use crate::semiring::*;
pub use crate::binary_op::*;
pub use crate::unary_op::*;

#[macro_use]
extern crate lazy_static;

#[test]
fn matrices_test() {
    let mut a = Matrix::<u32>::new(2, 2);
    a.build(&[0, 0, 1, 1], &[0, 1, 0, 1], &[1, 2, 3, 5], BinaryOp::<u32, u32, u32>::first());
    let mut b = Matrix::<u32>::new(2, 2);
    b.build(&[0, 0, 1, 1], &[0, 1, 0, 1], &[5, 3, 2, 1], BinaryOp::<u32, u32, u32>::first());

    let c = Matrix::<u32>::mxm(Semiring::<u32>::plus_times(), &a, &b);

    assert_eq!(c.get(0, 0), Some(9));
    assert_eq!(c.get(0, 1), Some(5));
    assert_eq!(c.get(1, 0), Some(25));
    assert_eq!(c.get(1, 1), Some(14));
}
