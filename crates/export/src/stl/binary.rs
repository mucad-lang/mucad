// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Binary STL writer

use std::io::Write;

use cgmath::{InnerSpace, Vector3};
use mucad_core::Triangle;

use crate::{Writer, stl::triangles::AsTriangleIterator};

/// Write into binary STL file
pub struct BinaryStlWriter<W: Write> {
    writer: W,
    header_written: bool,
    finished: bool,
    triangle_count: u32,
    /// Temporary buffer for triangle bytes to calculate total count before finalizing
    buffer: Vec<u8>,
}

impl<W: Write> BinaryStlWriter<W> {
    /// Create new binary STL writer
    pub fn new(w: W) -> Self {
        Self {
            writer: w,
            header_written: false,
            finished: false,
            triangle_count: 0,
            buffer: Vec::new(),
        }
    }

    fn write_triangle(&mut self, tri: &Triangle<Vector3<f32>>) -> std::io::Result<()> {
        let n = tri.normal().normalize();

        // 1. Facet Normal (3 x f32)
        self.buffer.extend_from_slice(&n.x.to_le_bytes());
        self.buffer.extend_from_slice(&n.y.to_le_bytes());
        self.buffer.extend_from_slice(&n.z.to_le_bytes());

        // 2. Vertex 0 (3 x f32)
        self.buffer.extend_from_slice(&tri.0.x.to_le_bytes());
        self.buffer.extend_from_slice(&tri.0.y.to_le_bytes());
        self.buffer.extend_from_slice(&tri.0.z.to_le_bytes());

        // 3. Vertex 1 (3 x f32)
        self.buffer.extend_from_slice(&tri.1.x.to_le_bytes());
        self.buffer.extend_from_slice(&tri.1.y.to_le_bytes());
        self.buffer.extend_from_slice(&tri.1.z.to_le_bytes());

        // 4. Vertex 2 (3 x f32)
        self.buffer.extend_from_slice(&tri.2.x.to_le_bytes());
        self.buffer.extend_from_slice(&tri.2.y.to_le_bytes());
        self.buffer.extend_from_slice(&tri.2.z.to_le_bytes());

        // 5. Attribute byte count (u16 = 0)
        self.buffer.extend_from_slice(&0u16.to_le_bytes());

        self.triangle_count += 1;
        Ok(())
    }
}

impl<W: Write> Writer for BinaryStlWriter<W> {
    type Output = W;

    fn begin(&mut self) -> std::io::Result<()> {
        if !self.header_written {
            // Write 80-byte header (zeroed)
            let header = [0u8; 80];
            self.writer.write_all(&header)?;
            self.header_written = true;
        }
        Ok(())
    }

    /// Writes header, total triangle count, buffered triangle data, and flushes output.
    fn finalize(&mut self) -> std::io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.begin()?;

        // Write total number of triangles (u32, Little Endian)
        self.writer.write_all(&self.triangle_count.to_le_bytes())?;

        // Write buffered triangle payloads
        self.writer.write_all(&self.buffer)?;
        self.writer.flush()?;

        self.finished = true;
        Ok(())
    }

    fn into_inner(mut self) -> std::io::Result<Self::Output> {
        self.finalize()?;
        Ok(self.writer)
    }
}

/// Trait to write something into a binary STL.
pub trait WriteBinaryStl {
    /// Write binary STL payload.
    fn write_binary_stl<W: Write>(&self, writer: &mut BinaryStlWriter<W>) -> std::io::Result<()>;
}

impl<T: AsTriangleIterator> WriteBinaryStl for T {
    fn write_binary_stl<W: Write>(&self, writer: &mut BinaryStlWriter<W>) -> std::io::Result<()> {
        self.triangle_iter()
            .try_for_each(|tri| writer.write_triangle(&tri))
    }
}
