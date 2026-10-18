// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import { createDockview, themeDark, type DockviewApi } from "dockview-core";

/** The local storage key of the saved layout. */
const LAYOUT_KEY = "mucad-web-viewer-layout";

/** A panel with its id, title and content. */
export interface Panel {
  id: string;
  title: string;
  element: HTMLElement;
}

/**
 * Dockable panels: tabs can be moved into other groups, split, floated and resized.
 *
 * The layout is saved in the local storage and restored on the next visit.
 * Panels cannot be closed, so the dock always shows all of them.
 */
export class Dock {
  private api: DockviewApi;

  constructor(
    element: HTMLElement,
    private panels: Panel[],
  ) {
    const elements = new Map(panels.map((panel) => [panel.id, panel.element]));
    this.api = createDockview(element, {
      theme: themeDark,
      createComponent: ({ id }) => ({ element: elements.get(id)!, init: () => {} }),
    });
    this.api.layout(element.clientWidth, element.clientHeight);

    if (!this.restore()) {
      this.reset();
    }
    this.api.onDidLayoutChange(() => {
      localStorage.setItem(LAYOUT_KEY, JSON.stringify(this.api.toJSON()));
    });
  }

  /**
   * The default layout: the first panel on the left, the second on the right and the others
   * below the first one.
   */
  reset() {
    localStorage.removeItem(LAYOUT_KEY);
    this.api.clear();

    const [first, second, ...others] = this.panels;
    this.add(first);
    this.add(second, { referencePanel: first.id, direction: "right" });
    for (const panel of others) {
      this.add(panel, { referencePanel: first.id, direction: "below" });
    }

    this.api.getPanel(first.id)?.api.setSize({ width: this.api.width * 0.45 });
    if (others.length) {
      this.api.getPanel(others[0].id)?.api.setSize({ height: this.api.height * 0.2 });
    }
  }

  /** Restore the saved layout, if it contains all panels. */
  private restore(): boolean {
    try {
      const layout = localStorage.getItem(LAYOUT_KEY);
      if (!layout) {
        return false;
      }
      this.api.fromJSON(JSON.parse(layout));
      return this.panels.every((panel) => this.api.getPanel(panel.id));
    } catch {
      return false;
    }
  }

  private add(panel: Panel, position?: { referencePanel: string; direction: "right" | "below" }) {
    this.api.addPanel({
      id: panel.id,
      title: panel.title,
      component: panel.id,
      position,
    });
  }
}
