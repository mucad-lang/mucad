// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! ASCII stl writer

use std::io::Write;

use cgmath::{InnerSpace, Vector3};
use mucad_core::Triangle;

use crate::{Writer, stl::triangles::AsTriangleIterator};

/// Write into STL file
pub struct AsciiStlWriter<W: Write> {
    writer: W,
    header_written: bool,
    finished: bool,
}

impl<W: Write> AsciiStlWriter<W> {
    /// Create new STL writer
    pub fn new(w: W) -> Self {
        Self {
            writer: w,
            header_written: false,
            finished: false,
        }
    }

    fn write_triangle(&mut self, tri: &Triangle<Vector3<f32>>) -> std::io::Result<()> {
        if !self.header_written {
            self.begin()?;
        }

        let n = tri.normal().normalize();
        writeln!(&mut self.writer, "facet normal {} {} {}", n.x, n.y, n.z)?;
        writeln!(&mut self.writer, "\touter loop")?;
        writeln!(
            &mut self.writer,
            "\t\tvertex {} {} {}",
            tri.0.x, tri.0.y, tri.0.z
        )?;
        writeln!(
            &mut self.writer,
            "\t\tvertex {} {} {}",
            tri.1.x, tri.1.y, tri.1.z
        )?;
        writeln!(
            &mut self.writer,
            "\t\tvertex {} {} {}",
            tri.2.x, tri.2.y, tri.2.z
        )?;
        writeln!(&mut self.writer, "\tendloop")?;
        writeln!(&mut self.writer, "endfacet")?;
        Ok(())
    }
}

impl<W: Write> Writer for AsciiStlWriter<W> {
    type Output = W;

    fn begin(&mut self) -> std::io::Result<()> {
        if !self.header_written {
            writeln!(&mut self.writer, "solid")?;
        }
        self.header_written = true;
        Ok(())
    }

    /// Flushes remaining content and writes final closing tags.
    fn finalize(&mut self) -> std::io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.begin()?;
        writeln!(self.writer, "endsolid")?;
        self.writer.flush()?;
        self.finished = true;
        Ok(())
    }

    fn into_inner(self) -> std::io::Result<Self::Output> {
        Ok(self.writer)
    }
}

/// Trait to write something into an ascii STL.
pub trait WriteAsciiStl {
    /// Write SVG tags.
    fn write_ascii_stl<W: Write>(&self, writer: &mut AsciiStlWriter<W>) -> std::io::Result<()>;
}

impl<T: AsTriangleIterator> WriteAsciiStl for T {
    fn write_ascii_stl<W: Write>(&self, writer: &mut AsciiStlWriter<W>) -> std::io::Result<()> {
        self.triangle_iter()
            .try_for_each(|tri| writer.write_triangle(&tri))
    }
}
