use std::{collections::HashMap, ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign}};

use crate::{eval::EvalScoreType, policy::PolicyScoreType};

pub mod eval;
pub mod policy;

#[derive(Debug, Default, Clone, PartialEq)]
struct SparseTrace {
    features: HashMap<u32, f32>,
}

impl SparseTrace {
    fn single(feature: u32) -> Self {
        Self {
            features: HashMap::from([(feature, 1.0)]),
        }
    }
}

impl AddAssign for SparseTrace {
    fn add_assign(&mut self, rhs: Self) {
        for (feature_idx, value) in rhs.features {
            self.features
                .entry(feature_idx)
                .and_modify(|e| *e += value)
                .or_insert(value);
        }
    }
}

impl Add for SparseTrace {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl SubAssign for SparseTrace {
    fn sub_assign(&mut self, rhs: Self) {
        for (feature_idx, value) in rhs.features {
            self.features
                .entry(feature_idx)
                .and_modify(|e| *e -= value)
                .or_insert(-value);
        }
    }
}

impl Sub for SparseTrace {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result -= rhs;
        result
    }
}

impl Neg for SparseTrace {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let mut result = self.clone();
        for value in result.features.values_mut() {
            *value = -*value;
        }
        result
    }
}

impl Mul<f32> for SparseTrace {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = self.clone();
        for value in result.features.values_mut() {
            *value *= rhs;
        }
        result
    }
}

impl Div<f32> for SparseTrace {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        let mut result = self.clone();
        for value in result.features.values_mut() {
            *value /= rhs;
        }
        result
    }
}

impl Mul<i32> for SparseTrace {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
		self * rhs as f32
    }
}

impl Div<i32> for SparseTrace {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
		self / rhs as f32
    }
}

impl EvalScoreType for SparseTrace {}
impl PolicyScoreType for SparseTrace {}
