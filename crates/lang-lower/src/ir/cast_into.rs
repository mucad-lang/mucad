// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! CastInto Trait

use mucad_lang_base::{Identifier, Refer, boxed};

pub trait CastInto<T> {
    fn cast_into(self) -> T;
}

impl<A, B> CastInto<Box<[B]>> for Box<[A]>
where
    A: CastInto<B>,
{
    fn cast_into(self) -> Box<[B]> {
        boxed(self.into_iter().map(|a| a.cast_into()))
    }
}

impl<A, B> CastInto<Box<B>> for Box<A>
where
    A: CastInto<B>,
    Box<B>: From<Box<A>>,
{
    fn cast_into(self) -> Box<B> {
        self.into()
    }
}

impl<A, B> CastInto<Refer<B>> for Refer<A>
where
    A: CastInto<B>,
{
    fn cast_into(self) -> Refer<B> {
        Refer::new(self.value.cast_into(), self.src_ref)
    }
}

impl CastInto<Identifier> for Identifier {
    #[inline]
    fn cast_into(self) -> Identifier {
        self
    }
}
