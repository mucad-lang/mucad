// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! STL test

use microcad_builtin::{BuiltinEvalContext, mu};
use microcad_core::Rect;
use microcad_export::{
    Export, Writer,
    stl::{AsciiStlWriter, StlExporter, WriteAsciiStl},
};
use microcad_lang_types::{ModelTree, Value, arguments, model::Element};
use microcad_render::Renderer;

use crate::common::{rect, target_dir};
mod common;

/// Renders STL content using a canvas closure, performs snapshot testing via `insta`,
/// and writes the output STL to disk under `target/svg_outputs/<snapshot_name>.stl`.
pub fn assert_stl_snapshot<F>(snapshot_name: &str, draw: F) -> std::io::Result<()>
where
    F: FnOnce(&mut AsciiStlWriter<Vec<u8>>) -> std::io::Result<()>,
{
    let mut writer = AsciiStlWriter::new(vec![]);
    draw(&mut writer)?;

    let buffer = writer.finish()?;

    Ok(insta::with_settings!(
        {
            prepend_module_to_snapshot => false,
            snapshot_path => "../snapshots",
        },
        {
            insta::assert_binary_snapshot!(format!("{snapshot_name}.stl").as_str(), buffer);
        }
    ))
}

/// Rect(0mm,0mm,10mm,10mm).extrude(10mm);
#[test]
fn test_cube() -> std::io::Result<()> {
    let mut ctx = BuiltinEvalContext::default();

    assert_stl_snapshot("test_cube", |writer| {
        let rect = rect(Rect::new((0., 0.), (10., 10.0)));
        let extrude = mu::ops::extrude(
            arguments!(
                self = rect,
                height = Value::mm(10.0),
                scale_x = Value::from(1.0),
                scale_y = Value::from(1.0),
                twist = Value::deg(0.0)
            ),
            &mut ctx,
        )
        .expect("No error");
        let geometry = Renderer::new().render(&extrude).expect("No render errors");

        geometry.write_ascii_stl(writer)?;
        Ok(())
    })
}

/// Export:
///
/// (Rect(0mm, 0mm, 100mm, 100mm) - Rect(0mm, 0mm, 100mm, 100mm).translate(50mm, 50mm)).extrude(10mm)
#[test]
fn stl_export() -> std::io::Result<()> {
    let bounds = Rect::new((0., 0.), (30., 30.));

    let mut ctx = BuiltinEvalContext::default();
    let mut group = ModelTree::new(Element::Group);

    let rect = rect(bounds);
    group.append(rect.clone());
    let translate = mu::ops::translate(
        arguments!(
            self = rect,
            x = Value::mm(10.0),
            y = Value::mm(10.0),
            z = Value::mm(0.0)
        ),
        &mut ctx,
    )
    .expect("No error");
    group.append(translate);

    let diff = mu::ops::difference(arguments!(self = group), &mut ctx).expect("No error");
    let extrude = mu::ops::extrude(
        arguments!(
            self = diff,
            height = Value::mm(3.0),
            scale_x = Value::from(1.0),
            scale_y = Value::from(1.0),
            twist = Value::deg(0.0)
        ),
        &mut ctx,
    )
    .expect("No error");

    // Test write ASCII STL
    StlExporter::default()
        .export_to_path(&extrude, &target_dir().join("rect_diff_ascii.stl"))
        .expect("No error");

    // Test write Binary STL
    StlExporter { binary: true }
        .export_to_path(&extrude, &target_dir().join("rect_diff_binary.stl"))
        .expect("No error");

    Ok(())
}
