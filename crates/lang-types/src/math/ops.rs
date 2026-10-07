// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Mathematical operations.

/// Mathematical operation trait for any value type.
pub trait MathOps {
    type Output;
    type Error;

    fn abs(&self) -> Result<Self::Output, Self::Error>;
    fn signum(&self) -> Result<Self::Output, Self::Error>;

    fn sin(&self) -> Result<Self::Output, Self::Error>;
    fn cos(&self) -> Result<Self::Output, Self::Error>;
    fn tan(&self) -> Result<Self::Output, Self::Error>;

    // Rounding & Conversions
    fn int(&self) -> Result<Self::Output, Self::Error>;
    fn floor(&self) -> Result<Self::Output, Self::Error>;
    fn ceil(&self) -> Result<Self::Output, Self::Error>;
    fn round(&self) -> Result<Self::Output, Self::Error>;
    fn fract(&self) -> Result<Self::Output, Self::Error>;

    fn sqrt(&self) -> Result<Self::Output, Self::Error>;
}
