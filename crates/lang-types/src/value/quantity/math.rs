// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Math functions for Quantity

use fixed::traits::FromFixed;

use crate::{Integer, MathOps, Quantity, Scalar, Value, ValueError, ValueResult};

impl MathOps for Quantity {
    type Output = Value;
    type Error = ValueError;

    fn sqrt(&self) -> ValueResult {
        if self.value < 0 {
            Err(ValueError::DomainError(
                "Cannot compute sqrt of negative number".into(),
            ))
        } else {
            Ok(self.clone().map(|v| v.sqrt()).into())
        }
    }

    fn abs(&self) -> ValueResult {
        Ok(self.clone().map(|v| v.abs()).into())
    }

    fn sin(&self) -> ValueResult {
        Ok(self
            .clone()
            .map(|v| Scalar::from_num(v.to_num::<f64>().sin()))
            .into())
    }

    fn cos(&self) -> Result<Self::Output, Self::Error> {
        Ok(self
            .clone()
            .map(|v| Scalar::from_num(v.to_num::<f64>().cos()))
            .into())
    }

    fn tan(&self) -> Result<Self::Output, Self::Error> {
        Ok(self
            .clone()
            .map(|v| Scalar::from_num(v.to_num::<f64>().tan()))
            .into())
    }

    fn int(&self) -> Result<Self::Output, Self::Error> {
        Ok(Integer::from_fixed(self.value).into())
    }

    fn signum(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.clone().map(|v| v.signum()).into())
    }

    fn floor(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.clone().map(|v| v.floor()).into())
    }

    fn ceil(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.clone().map(|v| v.ceil()).into())
    }

    fn round(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.clone().map(|v| v.round()).into())
    }

    fn fract(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.clone().map(|v| v.frac()).into())
    }
}
