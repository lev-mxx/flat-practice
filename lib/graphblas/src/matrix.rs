#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::*;
use std::mem::MaybeUninit;
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter};
use std::ptr;
use std::os::raw::c_void;

make_ffi_trait!(Matrix<T>);

pub struct BaseTypeMatrix<T> {
    link: GrbLink,
    _T: PhantomData<*const T>,
}

impl<T> Matrix<T> for BaseTypeMatrix<T> {
    fn grb_link(&self) -> &GrbLink { &self.link }

    fn grb_link_mut(&mut self) -> &mut GrbLink { &mut self.link }
}

impl<T: 'static + GrbType<T>> dyn Matrix<T> {
    pub fn new(rows: u64, cols: u64) -> BaseTypeMatrix<T> {
        let mat = grb_call!(GrB_Matrix_new, *mut c_void, T::grb_type().grb_link().link(), rows, cols);

        BaseTypeMatrix {
            link: GrbLink::of(mat),
            _T: PhantomData,
        }
    }

    pub fn mxm<S: Semiring<T>>(
        semiring: &S,
        a: &BaseTypeMatrix<T>,
        b: &BaseTypeMatrix<T>,
    ) -> BaseTypeMatrix<T> {
        let mut m = Matrix::new(a.nrows(), b.ncols());
        m.assign_mxm(semiring, a, b);
        m
    }

    pub fn kronecker<S: Semiring<T>>(
        semiring: &S,
        a: &BaseTypeMatrix<T>,
        b: &BaseTypeMatrix<T>,
    ) -> BaseTypeMatrix<T> {
        let mut m = Matrix::new(a.nrows() * b.nrows(), a.ncols() * b.ncols());
        m.assign_kronecker(semiring, a, b);
        m
    }
}

pub trait MatrixActions<T> {
    fn nrows(&self) -> u64;
    fn ncols(&self) -> u64;
    fn nvals(&self) -> u64;
    fn clear(&mut self);
    fn assign_mxm<S: Semiring<T>>(
        &mut self,
        semiring: &S,
        a: &Self,
        b: &Self,
    );

    fn accumulate_mxm<X, A: BinaryOp<T, X, T>, S: Semiring<X>, M: Matrix<X>>(
        &mut self,
        acc: &A,
        semiring: &S,
        a: &M,
        b: &M,
    );

    fn assign_apply<X, O: UnaryOp<X, T>, M: Matrix<X>>(
        &mut self,
        op: &O,
        a: &M,
    );

    fn accumulate_apply<X, Y, A: BinaryOp<T, Y, T>, O: UnaryOp<X, Y>, M: Matrix<X>>(
        &mut self,
        acc: &A,
        op: &O,
        a: &M,
    );

    fn assign_kronecker<S: Semiring<T>>(
        &mut self,
        semiring: &S,
        a: &Self,
        b: &Self,
    );

    fn accumulate_kronecker<X, S: Semiring<X>, A: BinaryOp<T, X, T>, M: Matrix<X>>(
        &mut self,
        acc: &A,
        semiring: &S,
        a: &M,
        b: &M,
    );
}

impl<T, MT: Matrix<T>> MatrixActions<T> for MT {
    fn nrows(&self) -> u64 {
        grb_call!(GrB_Matrix_nrows, u64, self.grb_link().link())
    }

    fn ncols(&self) -> u64 {
        grb_call!(GrB_Matrix_ncols, u64, self.grb_link().link())
    }

    fn nvals(&self) -> u64 {
        grb_call!(GrB_Matrix_nvals, u64, self.grb_link().link())
    }

    fn clear(&mut self) {
        grb_run!(GrB_Matrix_clear, self.grb_link_mut().link_mut());
    }

    fn assign_mxm<S: Semiring<T>>(
        &mut self,
        semiring: &S,
        a: &Self,
        b: &Self,
    ) {
        grb_run!(GrB_mxm, self.grb_link_mut().link_mut(), ptr::null_mut(), ptr::null_mut(), semiring.grb_link().link(), a.grb_link().link(), b.grb_link().link(), ptr::null_mut());
    }

    fn accumulate_mxm<X, A: BinaryOp<T, X, T>, S: Semiring<X>, M: Matrix<X>>(
        &mut self,
        acc: &A,
        semiring: &S,
        a: &M,
        b: &M,
    ) {
        grb_run!(GrB_mxm, self.grb_link_mut().link_mut(), ptr::null_mut(), acc.grb_link().link(), semiring.grb_link().link(), a.grb_link().link(), b.grb_link().link(), ptr::null_mut());
    }

    fn assign_apply<X, O: UnaryOp<X, T>, M: Matrix<X>>(
        &mut self,
        op: &O,
        a: &M,
    ) {
        grb_run!(GrB_Matrix_apply, self.grb_link_mut().link_mut(), ptr::null_mut(), ptr::null_mut(), op.grb_link().link(), a.grb_link().link(), ptr::null_mut());
    }

    fn accumulate_apply<X, Y, A: BinaryOp<T, Y, T>, O: UnaryOp<X, Y>, M: Matrix<X>>(
        &mut self,
        acc: &A,
        op: &O,
        a: &M,
    ) {
        grb_run!(GrB_Matrix_apply, self.grb_link_mut().link_mut(), ptr::null_mut(), acc.grb_link().link(), op.grb_link().link(), a.grb_link().link(), ptr::null_mut());
    }

    fn assign_kronecker<S: Semiring<T>>(
        &mut self,
        semiring: &S,
        a: &Self,
        b: &Self,
    ) {
        grb_run!(GrB_Matrix_kronecker_Semiring, self.grb_link_mut().link_mut(), ptr::null_mut(), ptr::null_mut(), semiring.grb_link().link(), a.grb_link().link(), b.grb_link().link(), ptr::null_mut());
    }

    fn accumulate_kronecker<X, S: Semiring<X>, A: BinaryOp<T, X, T>, M: Matrix<X>>(
        &mut self,
        acc: &A,
        semiring: &S,
        a: &M,
        b: &M,
    ) {
        grb_run!(GrB_Matrix_kronecker_Semiring, self.grb_link_mut().link_mut(), ptr::null_mut(), acc.grb_link().link(), semiring.grb_link().link(), a.grb_link().link(), b.grb_link().link(), ptr::null_mut());
    }
}

impl<T> Clone for BaseTypeMatrix<T> {
    fn clone(&self) -> Self {
        BaseTypeMatrix {
            link: GrbLink::of(grb_call!(GrB_Matrix_dup, *mut c_void, self.link.link())),
            _T: PhantomData,
        }
    }
}

impl<T> Drop for BaseTypeMatrix<T> {
    fn drop(&mut self) {
        grb_run!(GrB_Matrix_free, &mut self.link.link_mut());
    }
}

impl<T> Debug for BaseTypeMatrix<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (r, c) = (self.nrows(), self.ncols());
        let nvals = self.nvals();
        write!(
            f,
            "SparseMatrix[shape=({}x{}), vals={}]",
            r,
            c,
            nvals,
        )
    }
}

pub trait MatrixBuilder<T> {
    fn build<I: AsRef<[u64]>, J: AsRef<[u64]>, X: AsRef<[T]>, B: BinaryOp<T, T, T>>(&mut self, I: I, J: J, X: X, dup: &B) -> &mut Self;
}

macro_rules! make_matrix_builder {
    ( $rust_typ:ty, $grb_assign_fn:ident ) => {
        extern "C" {
            fn $grb_assign_fn(
                C: *mut c_void,
                I: *const u64,
                J: *const u64,
                X: *const $rust_typ,
                nvals: u64,
                dup: *const c_void,
            ) -> u32;
        }

        impl<M: Matrix<$rust_typ>> MatrixBuilder<$rust_typ> for M {
            fn build<I: AsRef<[u64]>, J: AsRef<[u64]>, X: AsRef<[$rust_typ]>, B: BinaryOp<$rust_typ, $rust_typ, $rust_typ>>(&mut self, I: I, J: J, X: X, dup: &B) -> &mut Self {
                let I = I.as_ref();
                let J = J.as_ref();
                let X = X.as_ref();

                let n = I.len() as u64;

                grb_run!($grb_assign_fn, self.grb_link_mut().link_mut(), I.as_ptr(), J.as_ptr(), X.as_ptr(), n, dup.grb_link().link());
                self
            }
        }
    };
}

macro_rules! make_builders{
    ( $rust_tpe:ty, $grb_tpy:ident ) => {
        paste::item! {
            make_matrix_builder!($rust_tpe, [<GrB_Matrix_build_ $grb_tpy>]);
        }
    }
}

for_each_type!(make_builders);

pub trait Get<T> {
    fn get(&self, i: u64, j: u64) -> Option<T>;
}

pub trait Insert<T> {
    fn insert(&mut self, i: u64, j: u64, value: T);
}

pub trait ExtractTuples<T> {
    fn extract_tuples(&self) -> (Vec<u64>, Vec<u64>, Vec<T>);
}

macro_rules! matrix_extract_impl {
    ( $ty:ty, $grb_ty:ident ) => {
        paste::paste! {
            _matrix_extract_impl!($ty, [<GrB_Matrix_extractTuples_ $grb_ty>]);
        }
    }
}

macro_rules! _matrix_extract_impl {
    ( $typ:ty, $extract_elem_func:ident ) => {

        extern "C" {
            fn $extract_elem_func(
                I: *mut u64,
                J: *mut u64,
                X: *mut $typ,
                nvals: *mut u64,
                A: *const c_void,
            ) -> u32;
        }

        impl<M: Matrix<$typ>> ExtractTuples<$typ> for M {
            fn extract_tuples(&self) -> (Vec<u64>, Vec<u64>, Vec<$typ>) {
                let size = self.nvals();
                let mut is = Vec::with_capacity(size as usize);
                let mut js = Vec::with_capacity(size as usize);
                let mut vs = Vec::with_capacity(size as usize);

                let mut nvals = self.nvals();

                grb_run!($extract_elem_func, is.as_mut_ptr(), js.as_mut_ptr(), vs.as_mut_ptr(), &mut nvals, self.grb_link().link());

                unsafe {
                    is.set_len(nvals as usize);
                    js.set_len(nvals as usize);
                    vs.set_len(nvals as usize);
                }

                (is, js, vs)
            }
        }
    };
}

macro_rules! matrix_insert_impl {
    ( $ty:ty, $grb_ty:ident ) => {
        paste::item! {
            _matrix_insert_impl!($ty, [<GrB_Matrix_setElement_ $grb_ty>]);
        }
    }
}

macro_rules! _matrix_insert_impl {
    ( $typ:ty, $set_elem_func:ident ) => {
         extern "C" {
            fn $set_elem_func(
                C: *mut c_void,
                x: $typ,
                i: u64,
                j: u64,
            ) -> u32;
         }

        impl<M: Matrix<$typ>> Insert<$typ> for M {
            fn insert(&mut self, i: u64, j: u64, val: $typ) {
                grb_run!($set_elem_func, self.grb_link_mut().link_mut(), val, i, j);
            }
        }
    };
}


macro_rules! matrix_get_impl {
    ( $ty:ty, $grb_ty:ident ) => {
        paste::paste! {
            _matrix_get_impl!($ty, [<GrB_Matrix_extractElement_ $grb_ty>]);
        }
    }
}

macro_rules! _matrix_get_impl {
    ( $typ:ty, $get_elem_func:ident ) => {
        extern "C" {
            fn $get_elem_func(
                x: *mut $typ,
                A: *const c_void,
                i: u64,
                j: u64,
            ) -> u32;
        }

        impl<M: Matrix<$typ>> Get<$typ> for M {
            fn get(&self, i: u64, j: u64) -> Option<$typ> {
                let mut P = MaybeUninit::<$typ>::uninit();
                unsafe {
                    match $get_elem_func(P.as_mut_ptr(), self.grb_link().link(), i, j) {
                        0 => Some(P.assume_init()),
                        1 => None,
                        e => panic!("Failed to get element at ({}, {}) GrB_error: {}", i, j, e),
                    }
                }
            }
        }
    };
}

for_each_type!(matrix_get_impl);
for_each_type!(matrix_insert_impl);
for_each_type!(matrix_extract_impl);

#[link(name = "graphblas")]
extern "C" {
    fn GrB_Matrix_new(
        A: *mut*mut c_void,
        type_: *const c_void,
        nrows: u64,
        ncols: u64,
    ) -> u32;

    fn GrB_Matrix_dup(
        C: *mut*mut c_void,
        A: *const c_void,
    ) -> u32;

    fn GrB_Matrix_clear(
        A: *mut c_void,
    ) -> u32;

    fn GrB_Matrix_nrows(
        nrows: *mut u64,
        A: *const c_void,
    ) -> u32;

    fn GrB_Matrix_ncols(
        ncols: *mut u64,
        A: *const c_void,
    ) -> u32;

    fn GrB_Matrix_nvals(
        nvals: *mut u64,
        A: *const c_void,
    ) -> u32;

    fn GrB_Matrix_free(
        A: *mut*mut c_void,
    ) -> u32;

    fn GrB_mxm(
        C: *mut c_void,
        Mask: *const c_void,
        accum: *const c_void,
        semiring: *const c_void,
        A: *const c_void,
        B: *const c_void,
        desc: *const c_void,
    ) -> u32;

    fn GrB_Matrix_apply (
        C: *mut c_void,
        Mask: *const c_void,
        accum: *const c_void,
        op: *const c_void,
        A: *const c_void,
        desc: *const c_void,
    ) -> u32;

    fn GrB_Matrix_kronecker_Semiring(
        C: *mut c_void,
        Mask: *const c_void,
        accum: *const c_void,
        semiring: *const c_void,
        A: *const c_void,
        B: *const c_void,
        desc: *const c_void,
    ) -> u32;
}
