#![allow(unused)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use derive_more::Deref;
use derive_more::DerefMut;
use macros::DeepClone;
use ndarray::ArrayBase;
use ndarray::Axis;
use ndarray::Dim;
use ndarray::IxDyn;
use ndarray::IxDynImpl;
use ndarray::OwnedRepr;
use ndarray::SliceInfo;
use ndarray::SliceInfoElem;
use num::Num;
use num::Zero;
use serde::Deserialize;
use serde::Serialize;

use crate::array::Array;
use crate::cow::Cow;
use crate::iterator::Iter;
use crate::traits::Data;
use crate::traits::DeepClone;
use crate::vec::Vec;

#[derive(Debug, DeepClone, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub struct Matrix<T>(pub Cow<Inner<T>>);

#[derive(Clone, Deref, DerefMut, Serialize, Deserialize, Eq, PartialEq)]
#[repr(C)]
pub struct Inner<T>(pub ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>);

impl<T: std::fmt::Debug> std::fmt::Debug for Inner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Clone> DeepClone for Inner<T> {
    fn deep_clone(&self) -> Self {
        Inner(self.0.clone())
    }
}

impl<T> Matrix<T> {
    pub fn zeros(shape: Vec<usize>) -> Self
    where
        T: Clone + Zero,
    {
        Matrix::from(ArrayBase::zeros(shape.0.take()))
    }

    pub fn insert_axis(mut self, axis: usize) -> Self
    where
        T: Clone,
    {
        self.0.update(|this| this.0.insert_axis_inplace(Axis(axis)));
        self
    }

    pub fn remove_axis(mut self, axis: usize) -> Self
    where
        T: Clone,
    {
        self.0
            .map(|this| Matrix::from(this.0.remove_axis(Axis(axis))))
    }

    pub fn into_vec(self) -> Vec<T>
    where
        T: Clone,
    {
        self.0.map(|this| Vec::from(this.0.into_raw_vec()))
    }

    pub fn iter(self) -> Iter<impl Iterator<Item = T>>
    where
        T: Clone,
    {
        Iter::new(self.0.take().0.into_raw_vec().into_iter())
    }

    pub fn transpose(self) -> Self
    where
        T: Clone,
    {
        Matrix::from(self.0.map(|this| this.0.t().into_owned()))
    }

    // pub fn slice<const N: usize>(self, indices: Array<Index, N>) -> Self
    // where
    //     T: Clone,
    // {
    //     Matrix::from(self.0.map(|this| {
    //         let x: SliceInfo<std::vec::Vec<SliceInfoElem>, IxDyn, IxDyn> =
    //             SliceInfo::try_from(vec![SliceInfoElem::from(1..)]).unwrap();
    //         this.0.slice(x).into_owned()
    //     }))
    // }
}

impl<T> From<ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>> for Matrix<T> {
    fn from(array: ArrayBase<OwnedRepr<T>, Dim<IxDynImpl>>) -> Self {
        Matrix(Cow::new(Inner(array)))
    }
}

/// [a]
/// [:]
/// [a:b:2]
/// [a::-1]
/// [np.newaxis]
#[derive(Debug, DeepClone, Clone, Serialize, Deserialize)]
pub enum Index {
    Range(usize, usize),
    Index(usize),
}
