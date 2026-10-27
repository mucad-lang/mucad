// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import {
  createDockview,
  createPaneview,
  themeDark,
  type DockviewApi,
  type PaneviewApi,
} from "dockview-core";

/** A view or subscription, which is disposed when the screen width changes. */
type Disposable = { dispose(): void };

/** The local storage key of the saved layout. */
const LAYOUT_KEY = "mucad-web-viewer-layout";

/** Narrow screens, e.g. phones, which show one panel at a time. */
const MOBILE = window.matchMedia("(max-width: 768px)");

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
 *
 * On narrow screens, the panels are a vertical stack instead, of which one is expanded at a
 * time, and the saved layout is kept for wide screens. The stack starts with `stackFirst`,
 * which is expanded at first, e.g. the view of the model.
 */
export class Dock {
  /** The panels as dock on wide screens, `undefined` on narrow screens. */
  private api?: DockviewApi;
  /** The view and its subscriptions, which are disposed when the screen width changes. */
  private disposables: Disposable[] = [];
  /** The expanded panel on narrow screens. */
  private expanded: string;

  constructor(
    private element: HTMLElement,
    private panels: Panel[],
    private stackFirst = panels[0].id,
  ) {
    this.expanded = stackFirst;
    this.create();
    MOBILE.addEventListener("change", () => {
      this.dispose();
      this.create();
    });
  }

  /**
   * The default layout: the first panel on the left, the second on the right and the others
   * below the first one.
   */
  reset() {
    localStorage.removeItem(LAYOUT_KEY);
    const api = this.api;
    if (!api) return;
    api.clear();

    const [first, second, ...others] = this.panels;
    this.add(api, first);
    this.add(api, second, { referencePanel: first.id, direction: "right" });
    for (const panel of others) {
      this.add(api, panel, { referencePanel: first.id, direction: "below" });
    }

    api.getPanel(first.id)?.api.setSize({ width: api.width * 0.45 });
    if (others.length) {
      api.getPanel(others[0].id)?.api.setSize({ height: api.height * 0.2 });
    }
  }

  /** Create the view for the screen width. */
  private create() {
    if (MOBILE.matches) this.createStack();
    else this.createDock();
  }

  /** Dispose the view and its subscriptions, the panel contents are kept for the next view. */
  private dispose() {
    // The subscriptions first, so that the disposed dock does not save an empty layout.
    for (const disposable of this.disposables.reverse()) disposable.dispose();
    this.disposables = [];
    this.api = undefined;
  }

  /** The content of a panel in a view. */
  private component(id: string) {
    const element = this.panels.find((panel) => panel.id === id)!.element;
    // The content is kept, when the view is disposed.
    return { element, init: () => {}, update: () => {}, dispose: () => {} };
  }

  /** The dock of wide screens with the saved layout. */
  private createDock() {
    const api = createDockview(this.element, {
      theme: themeDark,
      createComponent: ({ id }) => this.component(id),
    });
    this.api = api;
    this.disposables.push(api);
    api.layout(this.element.clientWidth, this.element.clientHeight);

    if (!this.restore(api)) {
      this.reset();
    }
    this.disposables.push(
      api.onDidLayoutChange(() => {
        localStorage.setItem(LAYOUT_KEY, JSON.stringify(api.toJSON()));
      }),
    );
  }

  /** The vertical stack of narrow screens, of which one panel is expanded at a time. */
  private createStack() {
    const api: PaneviewApi = createPaneview(this.element, {
      className: "dockview-theme-dark",
      disableDnd: true,
      createComponent: ({ id }) => this.component(id),
    });
    this.disposables.push(api);
    const stack = [
      ...this.panels.filter((panel) => panel.id === this.stackFirst),
      ...this.panels.filter((panel) => panel.id !== this.stackFirst),
    ];
    for (const panel of stack) {
      const pane = api.addPanel({
        id: panel.id,
        title: panel.title,
        component: panel.id,
        isExpanded: panel.id === this.expanded,
      });
      this.disposables.push(
        pane.api.onDidExpansionChange(({ isExpanded }) => {
          if (isExpanded) {
            this.expanded = panel.id;
            for (const other of api.panels) {
              if (other.id !== panel.id) other.api.setExpanded(false);
            }
          } else if (this.expanded === panel.id) {
            // One panel stays expanded, another one is expanded by its header.
            pane.api.setExpanded(true);
          }
        }),
      );
    }
    api.layout(this.element.clientWidth, this.element.clientHeight);
  }

  /** Restore the saved layout, if it contains all panels. */
  private restore(api: DockviewApi): boolean {
    try {
      const layout = localStorage.getItem(LAYOUT_KEY);
      if (!layout) {
        return false;
      }
      api.fromJSON(JSON.parse(layout));
      return this.panels.every((panel) => api.getPanel(panel.id));
    } catch {
      return false;
    }
  }

  private add(
    api: DockviewApi,
    panel: Panel,
    position?: { referencePanel: string; direction: "right" | "below" },
  ) {
    api.addPanel({
      id: panel.id,
      title: panel.title,
      component: panel.id,
      position,
    });
  }
}
