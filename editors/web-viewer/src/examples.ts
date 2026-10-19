// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

/** An example from the repository's `examples` folder. */
export interface Example {
  name: string;
  code: string;
}

// Fetch all .mu files as raw strings eagerly
const modules = import.meta.glob<{ [key: string]: string } | string>(
  '../../../examples/*.mu',
  { eager: true, query: '?raw', import: 'default' }
);

// Transform the module dictionary into an Example array dynamically
export const examples: Example[] = Object.entries(modules).map(([path, code]) => {
  // Extract file name from path (e.g., "../../examples/csg_cube.mu" -> "csg_cube")
  const name = path.split('/').pop()?.replace('.mu', '') || '';

  return {
    name,
    code: code as string,
  };
});
