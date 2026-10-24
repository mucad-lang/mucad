// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { toCreasedNormals } from "three/examples/jsm/utils/BufferGeometryUtils.js";
import type { Mesh, Sketch } from "mucad-wasm";
import { theme } from "./theme";

/** The angle between faces, above which an edge is shown sharp instead of smooth. */
const CREASE_ANGLE = THREE.MathUtils.degToRad(20);

/** The directions to look at 3D models (from the front at an angle) and 2D models (from above). */
const VIEW_DIRECTION = {
  "3d": new THREE.Vector3(1, -1.5, 1),
  "2d": new THREE.Vector3(0, 0, 1),
} as const;

type ModelKind = keyof typeof VIEW_DIRECTION;

/** A three.js view of µcad models, the Z axis points up like in µcad. */
export class Viewer {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100000);
  private controls: OrbitControls;
  private model = new THREE.Group();
  private grid?: THREE.GridHelper;
  /** The kind of the shown model, whose direction the camera was reset to. */
  private kind?: ModelKind;

  constructor(private element: HTMLElement) {
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    element.appendChild(this.renderer.domElement);

    this.scene.background = new THREE.Color(theme.primary);
    this.scene.add(this.model);
    this.scene.add(new THREE.HemisphereLight(theme.bright, theme.darker, 1.5));
    const light = new THREE.DirectionalLight(theme.bright, 2);
    light.position.set(1, -2, 3);
    this.camera.add(light);
    this.scene.add(this.camera);

    THREE.Object3D.DEFAULT_UP.set(0, 0, 1);
    this.camera.up.set(0, 0, 1);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);

    const resetButton = document.createElement("button");
    resetButton.className = "view-button";
    resetButton.textContent = "Reset camera";
    resetButton.title = "Look at the whole model";
    resetButton.addEventListener("click", () => this.resetCamera());
    element.appendChild(resetButton);

    new ResizeObserver(() => this.resize()).observe(element);
    this.renderer.setAnimationLoop(() => {
      this.controls.update();
      this.renderer.render(this.scene, this.camera);
    });
  }

  /** Show 3D triangle meshes in their colors with contours at their sharp edges. */
  show3d(meshes: Mesh[]) {
    this.replace(
      meshes.flatMap((mesh) => {
        const material = new THREE.MeshStandardMaterial({
          ...colorParameters(mesh.color, theme.brighter),
          metalness: 0.1,
          roughness: 0.6,
          // Move the faces back a little, so that the contours on their edges stay visible.
          polygonOffset: true,
          polygonOffsetFactor: 1,
          polygonOffsetUnits: 1,
        });
        const surface = new THREE.Mesh(toCreasedNormals(geometry(mesh), CREASE_ANGLE), material);
        const contours = new THREE.LineSegments(
          new THREE.BufferGeometry()
            .setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3))
            .setIndex(new THREE.BufferAttribute(mesh.edges(), 1)),
          new THREE.LineBasicMaterial({ color: theme.primary }),
        );
        return [surface, contours];
      }),
    );
    this.update("3d");
  }

  /**
   * Show 2D sketches as filled triangles with outlines.
   *
   * Sketches without a color are filled in the darker color with outlines in the signal color.
   */
  show2d(sketches: Sketch[]) {
    this.replace(
      sketches.flatMap((sketch) => {
        const fill = new THREE.Mesh(
          geometry(sketch.triangles),
          new THREE.MeshBasicMaterial({
            ...colorParameters(sketch.color, theme.darker),
            side: THREE.DoubleSide,
          }),
        );
        const positions = sketch.lines.positions;
        const points = new Float32Array((positions.length / 2) * 3);
        positions.forEach(
          (value, index) => (points[Math.floor(index / 2) * 3 + (index % 2)] = value),
        );
        const outline = new THREE.LineSegments(
          new THREE.BufferGeometry().setAttribute("position", new THREE.BufferAttribute(points, 3)),
          new THREE.LineBasicMaterial(colorParameters(sketch.color, theme.signal)),
        );
        return [fill, outline];
      }),
    );
    this.update("2d");
  }

  /** Remove the model. */
  clear() {
    this.replace([]);
  }

  private replace(objects: THREE.Object3D[]) {
    this.model.children.forEach((child) => disposeObject(child));
    this.model.clear();
    objects.forEach((object) => this.model.add(object));
  }

  /**
   * Update the grid for a new model of `kind`, the camera stays, unless it is the first model or
   * the kind changes, e.g. from a 3D to a 2D model.
   */
  private update(kind: ModelKind) {
    const box = this.bounds();
    if (!box) return;
    if (this.kind !== kind) {
      this.kind = kind;
      this.resetCamera();
    } else {
      // A bigger model is not cut off.
      const distance = this.camera.position.distanceTo(this.controls.target);
      const radius = box.getBoundingSphere(new THREE.Sphere()).radius;
      this.camera.far = Math.max(this.camera.far, (distance + 2 * radius) * 10);
      this.camera.updateProjectionMatrix();
    }
    this.updateGrid(box);
  }

  /** Move the camera to see the whole model from the direction of its kind. */
  resetCamera() {
    const box = this.bounds();
    if (!box || !this.kind) return;
    const center = box.getCenter(new THREE.Vector3());
    const radius = box.getBoundingSphere(new THREE.Sphere()).radius || 1;
    const distance = radius / Math.sin(THREE.MathUtils.degToRad(this.camera.fov / 2));
    const direction = VIEW_DIRECTION[this.kind].clone().normalize();

    this.camera.position.copy(center).addScaledVector(direction, distance);
    this.camera.near = distance / 100;
    this.camera.far = distance * 100;
    this.camera.updateProjectionMatrix();
    this.controls.target.copy(center);
  }

  /** The bounds of the model, `undefined` without model. */
  private bounds(): THREE.Box3 | undefined {
    const box = new THREE.Box3().setFromObject(this.model);
    return box.isEmpty() ? undefined : box;
  }

  /** Add a grid below the model in `box`. */
  private updateGrid(box: THREE.Box3) {
    const center = box.getCenter(new THREE.Vector3());
    const radius = box.getBoundingSphere(new THREE.Sphere()).radius || 1;
    if (this.grid) {
      this.scene.remove(this.grid);
      this.grid.dispose();
    }
    // Cells of a power of ten, e.g. 10mm for a model of 50mm.
    const cell = Math.pow(10, Math.floor(Math.log10(radius / 2)));
    const divisions = 2 * Math.ceil((radius * 1.5) / cell);
    this.grid = new THREE.GridHelper(divisions * cell, divisions, theme.guide, theme.darker);
    this.grid.rotation.x = Math.PI / 2;
    this.grid.position.set(center.x, center.y, box.min.z - radius / 1000);
    this.scene.add(this.grid);
  }

  private resize() {
    const { clientWidth: width, clientHeight: height } = this.element;
    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    // Resizing clears the canvas, render at once to avoid flickering while a panel is resized.
    this.renderer.render(this.scene, this.camera);
  }
}

/**
 * Material parameters of a µcad color (`r, g, b, a` from 0 to 1), e.g. of `#[color = "red"]`,
 * or of the `fallback` color, if there is none.
 */
function colorParameters(color: Float32Array | undefined, fallback: string) {
  if (!color) return { color: fallback };
  const [r, g, b, a] = color;
  return {
    color: new THREE.Color().setRGB(r, g, b, THREE.SRGBColorSpace),
    transparent: a < 1,
    opacity: a,
  };
}

/** A three.js geometry of a µcad mesh. */
function geometry(mesh: Mesh): THREE.BufferGeometry {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.BufferAttribute(mesh.positions, 3));
  geometry.setIndex(new THREE.BufferAttribute(mesh.indices, 1));
  geometry.computeVertexNormals();
  return geometry;
}

function disposeObject(object: THREE.Object3D) {
  if (object instanceof THREE.Mesh || object instanceof THREE.LineSegments) {
    object.geometry.dispose();
    (object.material as THREE.Material).dispose();
  }
}
