// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Scaffolding builds the tree from desugared IR items.

use microcad_lang_base::{
    PushIssue, SingleIdentifier, SrcReferrer, Stability, Version, VersionAnnotation,
};
use microcad_lang_types::Value;

use crate::{LowerContext, LowerError, ir};

pub trait Scaffold: Sized {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId;
}

impl<T> Scaffold for Box<[T]>
where
    T: Scaffold,
{
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        self.into_iter().for_each(|item| {
            context
                .top_node()
                .append(item.scaffold(context), &mut context.arena)
        });
        *context.top_node()
    }
}

impl Scaffold for ir::desugared::FileModule {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item(ir::Item {
            meta: self.meta,
            def: ir::FileModule {}.into(),
            doc: self.attr.doc,
            ver,
        })
    }
}

impl Scaffold for ir::desugared::Alias {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item(ir::Item {
            meta: self.meta,
            def: ir::Alias { path: self.path }.into(),
            doc: self.attr.doc,
            ver,
        })
    }
}

impl Scaffold for ir::desugared::Wildcard {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item(ir::Item {
            meta: self.meta,
            def: ir::Wildcard { path: self.path }.into(),
            doc: self.attr.doc,
            ver,
        })
    }
}

impl Scaffold for ir::desugared::Constant {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item(ir::Item {
            meta: self.meta,
            def: ir::Constant {
                ty: self.ty,
                expr: self.expr,
            }
            .into(),
            doc: self.attr.doc,
            ver,
        })
    }
}

impl Scaffold for ir::desugared::InlineModule {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item_with_children(
            ir::Item {
                meta: self.meta,
                def: ir::InlineModule {}.into(),
                doc: self.attr.doc,
                ver,
            },
            self.items,
        )
    }
}

impl Scaffold for ir::desugared::Function {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item_with_children(
            ir::Item {
                meta: self.meta,
                def: ir::Function {
                    statements: self.statements,
                    signature: self.signature,
                }
                .into(),
                doc: self.attr.doc,
                ver,
            },
            self.items,
        )
    }
}

impl Scaffold for ir::desugared::Workbench {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item_with_children(
            ir::Item {
                meta: self.meta,
                def: ir::Workbench {
                    statements: self.statements,
                    signature: ir::WorkbenchSignature {
                        kind: self.kind,
                        parameters: self.parameters,
                        inits: self.inits,
                    },
                }
                .into(),
                doc: self.attr.doc,
                ver,
            },
            self.items,
        )
    }
}

impl ir::Attributes {
    /// Extracts a [`VersionAnnotation`] from the attribute list.
    ///
    /// Evaluates `#[introduced]`, `#[stable]`, `#[experimental]`, and `#[deprecated]` attributes.
    /// Any parsing errors, duplicate attributes, or conflicting stability declarations
    /// are recorded into `ctx` via `push_diag`.
    ///
    /// #[introduced(version = "0.2.0")]
    /// #[stable]
    /// #[experimental]
    /// #[deprecated]
    /// #[deprecated(since = "0.5.0", note = "Use ... instead", removal_in = "0.6.0")]
    pub fn fetch_ver(&self, ctx: &mut LowerContext) -> VersionAnnotation {
        /// Sets the stability state, emitting a diagnostic if stability was already declared.
        fn set_stability(
            slot: &mut Option<Stability>,
            new_stability: Stability,
            src_ref: impl SrcReferrer,
            ctx: &mut LowerContext,
        ) {
            if slot.is_some() {
                ctx.push_err(LowerError::ConflictingStability {
                    src_ref: src_ref.src_ref(),
                });
            } else {
                *slot = Some(new_stability);
            }
        }

        let mut introduced: Option<Version> = None;
        let mut stability: Option<Stability> = None;

        // 1. Process #[tags] (e.g., #[stable], #[experimental], #[deprecated])
        for tag in self.tags.iter() {
            let tag_name = tag.name.as_str();
            match tag_name {
                "stable" => {
                    set_stability(&mut stability, Stability::Stable, tag, ctx);
                }
                "experimental" => {
                    set_stability(&mut stability, Stability::Experimental, tag, ctx);
                }
                "deprecated" => {
                    set_stability(
                        &mut stability,
                        Stability::Deprecated {
                            since: None,
                            note: None,
                            removal_in: None,
                        },
                        tag,
                        ctx,
                    );
                }
                _ => {}
            }
        }

        // 2. Process #[commands] (e.g., #[introduced(...)], #[deprecated(...)])
        for cmd in self.commands.iter() {
            let cmd_name = match cmd.path.single_identifier() {
                Some(ident) => ident.as_str(),
                None => continue,
            };

            match cmd_name {
                "introduced" => {
                    let value = cmd.argument_list.extract_arg_value("version");
                    let ver_str = Self::value_as_string(value, ctx);
                    if let Some(ver_str) = ver_str {
                        match ver_str.parse::<Version>() {
                            Ok(ver) => {
                                if introduced.is_some() {
                                    ctx.push_err(LowerError::DuplicateAttribute {
                                        attr: "introduced".to_string(),
                                        src_ref: cmd.src_ref,
                                    });
                                } else {
                                    introduced = Some(ver);
                                }
                            }
                            Err(_) => {
                                ctx.push_err(LowerError::InvalidVersionString {
                                    value: ver_str,
                                    src_ref: cmd.src_ref,
                                });
                            }
                        }
                    } else {
                        ctx.push_err(LowerError::MissingAttributeArgument {
                            attr: "introduced".to_string(),
                            arg: "version".to_string(),
                            src_ref: cmd.src_ref,
                        });
                    }
                }
                "deprecated" => {
                    let since = cmd.argument_list.extract_arg_value("since");
                    let since =
                        Self::value_as_string(since, ctx).and_then(|s| s.parse::<Version>().ok());

                    let note = cmd.argument_list.extract_arg_value("note");
                    let note = Self::value_as_string(note, ctx);

                    let removal_in = cmd.argument_list.extract_arg_value("removal_in");
                    let removal_in = Self::value_as_string(removal_in, ctx)
                        .and_then(|s| s.parse::<Version>().ok());

                    let dep = Stability::Deprecated {
                        since,
                        note,
                        removal_in,
                    };
                    set_stability(&mut stability, dep, cmd.src_ref, ctx);
                }
                _ => {}
            }
        }

        VersionAnnotation {
            introduced,
            stability: stability.unwrap_or(Stability::Stable),
        }
    }

    fn value_as_string(value: Option<&Value>, _ctx: &mut LowerContext) -> Option<String> {
        match value {
            Some(Value::String(string)) => Some(string.to_string()),
            _ => None,
        }
    }
}

impl Scaffold for ir::desugared::Source {
    fn scaffold(self, context: &mut LowerContext) -> ir::NodeId {
        let ver = self.attr.fetch_ver(context);

        context.scaffold_item_with_children(
            ir::Item {
                meta: self.meta,
                def: ir::Source {
                    statements: self.statements,
                }
                .into(),
                doc: self.attr.doc,
                ver,
            },
            self.items,
        )
    }
}
