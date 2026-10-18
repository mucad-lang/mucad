// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { toCreasedNormals } from "three/examples/jsm/utils/BufferGeometryUtils.js";
import type { Lines, Mesh } from "mucad-wasm";
import { theme } from "./theme";

/** The angle between faces, above which an edge is shown sharp instead of smooth. */
const CREASE_ANGLE = THREE.MathUtils.degToRad(20);

/** A three.js view of µcad models, the Z axis points up like in µcad. */
export class Viewer {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100000);
  private controls: OrbitControls;
  private model = new THREE.Group();
  private grid?: THREE.GridHelper;

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

    new ResizeObserver(() => this.resize()).observe(element);
    this.renderer.setAnimationLoop(() => {
      this.controls.update();
      this.renderer.render(this.scene, this.camera);
    });
  }

  /** Show a 3D triangle mesh. */
  show3d(mesh: Mesh) {
    const material = new THREE.MeshStandardMaterial({
      color: theme.brighter,
      metalness: 0.1,
      roughness: 0.6,
    });
    this.replace([new THREE.Mesh(toCreasedNormals(geometry(mesh), CREASE_ANGLE), material)]);
    this.fit(new THREE.Vector3(1, -1.5, 1));
  }

  /** Show a 2D model as filled triangles with outlines. */
  show2d(triangles: Mesh, lines: Lines) {
    const fill = new THREE.Mesh(
      geometry(triangles),
      new THREE.MeshBasicMaterial({ color: theme.darker, side: THREE.DoubleSide }),
    );
    const positions = lines.positions;
    const points = new Float32Array((positions.length / 2) * 3);
    positions.forEach((value, index) => (points[Math.floor(index / 2) * 3 + (index % 2)] = value));
    const outline = new THREE.LineSegments(
      new THREE.BufferGeometry().setAttribute("position", new THREE.BufferAttribute(points, 3)),
      new THREE.LineBasicMaterial({ color: theme.signal }),
    );
    this.replace([fill, outline]);
    this.fit(new THREE.Vector3(0, 0, 1));
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

  /** Move the camera to see the whole model from `direction` and add a grid below it. */
  private fit(direction: THREE.Vector3) {
    const box = new THREE.Box3().setFromObject(this.model);
    if (box.isEmpty()) return;
    const center = box.getCenter(new THREE.Vector3());
    const radius = box.getBoundingSphere(new THREE.Sphere()).radius || 1;
    const distance = radius / Math.sin(THREE.MathUtils.degToRad(this.camera.fov / 2));

    this.camera.position.copy(center).addScaledVector(direction.normalize(), distance);
    this.camera.near = distance / 100;
    this.camera.far = distance * 100;
    this.camera.updateProjectionMatrix();
    this.controls.target.copy(center);

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
  }
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
