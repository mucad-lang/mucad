// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Keywords in the µcad languages. Keywords cannot be used as identifiers.

/// Declare the keyword enum from a table of its variants and their texts.
///
/// Reserved keywords are marked with `#[reserved]`. The macro adds the derives, a doc comment
/// with the text to each variant, the private tables `TEXTS` and `RESERVED` in the order of the
/// variants and [`std::str::FromStr`].
macro_rules! keywords {
    (@reserved reserved) => { true };
    (@reserved) => { false };
    (
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $( $(#[$flag:ident])? $variant:ident = $text:literal, )*
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $( #[doc = concat!("`", $text, "`")] $variant, )*
        }

        impl $name {
            /// The texts of the keywords in the order of the variants.
            const TEXTS: &[&str] = &[$($text,)*];
            /// If the keywords are reserved, in the order of the variants.
            const RESERVED: &[bool] = &[$(keywords!(@reserved $($flag)?),)*];
        }

        impl std::str::FromStr for $name {
            type Err = ();

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                match text {
                    $($text => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }
    };
}

keywords! {
    /// A keyword, e.g. `fn`, which cannot be used as an identifier.
    pub enum Keyword {
        Mod = "mod",
        Part = "part",
        Sketch = "sketch",
        Op = "op",
        Fn = "fn",
        If = "if",
        Else = "else",
        Use = "use",
        As = "as",
        Return = "return",
        Pub = "pub",
        Const = "const",
        Prop = "prop",
        Init = "init",
        Super = "super",
        #[reserved] Plugin = "__plugin",
        #[reserved] Assembly = "assembly",
        #[reserved] Material = "material",
        #[reserved] Unit = "unit",
        #[reserved] Enum = "enum",
        #[reserved] Struct = "struct",
        #[reserved] Match = "match",
        #[reserved] Type = "type",
        #[reserved] Extern = "extern",
    }
}

impl Keyword {
    /// The text of the keyword, e.g. `fn`.
    pub fn as_str(self) -> &'static str {
        Self::TEXTS[self as usize]
    }

    /// Returns if the keyword is reserved for the future, e.g. `enum`:
    /// it has no meaning yet, but cannot be used as an identifier.
    pub fn is_reserved(self) -> bool {
        Self::RESERVED[self as usize]
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Keyword;

    /// The text of every keyword is parsed into the keyword again.
    #[test]
    fn keyword_texts() {
        assert_eq!(Keyword::TEXTS.len(), Keyword::RESERVED.len());
        for text in Keyword::TEXTS {
            let keyword: Keyword = text.parse().expect("A keyword");
            assert_eq!(keyword.as_str(), *text);
        }
        assert_eq!("fn".parse(), Ok(Keyword::Fn));
        assert!(!Keyword::Fn.is_reserved());
        assert!(Keyword::Enum.is_reserved());
        assert_eq!("function".parse::<Keyword>(), Err(()));
    }
}
