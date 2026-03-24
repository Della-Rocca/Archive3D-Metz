<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { tick } from "svelte";
  import { onDestroy, onMount } from "svelte";
  import * as THREE from "three";

  export let modelPath: string | null = null;
  export let noVisualAssets = false;

  let viewerShellEl: HTMLElement | null = null;
  let mountEl: HTMLDivElement | null = null;
  let renderer: THREE.WebGLRenderer | null = null;
  let camera: THREE.PerspectiveCamera | null = null;
  let scene: THREE.Scene | null = null;
  let controls: { update: () => void; dispose: () => void } | null = null;
  let rootModel: THREE.Object3D | null = null;
  let frameId = 0;
  let disposed = false;
  let mounted = false;
  let lastInputSignature = "";
  let isExpanded = false;
  let status: "idle" | "loading" | "error" | "empty" = "idle";
  let statusMessage = "Sélectionnez une structure pour afficher son modèle 3D.";

  function fileExtension(path: string): string {
    const normalized = path.replace(/\\/g, "/");
    const name = normalized.split("/").pop() ?? normalized;
    const dotIndex = name.lastIndexOf(".");
    return dotIndex >= 0 ? name.slice(dotIndex + 1).toLowerCase() : "";
  }

  function toModelUrl(path: string): string {
    if (/^(https?:|tauri:|asset:|file:|blob:)/.test(path)) return path;
    return convertFileSrc(path.replace(/\\/g, "/"));
  }

  function setStatus(next: typeof status, message: string) {
    status = next;
    statusMessage = message;
  }

  function clearCurrentModel() {
    if (!scene || !rootModel) return;
    scene.remove(rootModel);
    rootModel.traverse((obj) => {
      const mesh = obj as THREE.Mesh;
      if (mesh.geometry) mesh.geometry.dispose();
      if (mesh.material) {
        const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
        materials.forEach((mat) => mat.dispose());
      }
    });
    rootModel = null;
  }

  function centerCameraOnObject(object: THREE.Object3D) {
    if (!camera || !controls) return;
    const box = new THREE.Box3().setFromObject(object);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    const maxDim = Math.max(size.x, size.y, size.z) || 1;
    const fov = camera.fov * (Math.PI / 180);
    const distance = maxDim / (2 * Math.tan(fov / 2));

    camera.position.set(center.x + distance * 1.1, center.y + distance * 0.6, center.z + distance * 1.1);
    camera.near = Math.max(distance / 100, 0.01);
    camera.far = distance * 100;
    camera.lookAt(center);
    camera.updateProjectionMatrix();
    (controls as any).target.copy(center);
    controls.update();
  }

  async function loadModel(path: string) {
    if (!scene) return;
    const source = path.trim();
    if (!source) {
      clearCurrentModel();
      setStatus("empty", "Aucun modèle 3D détecté.");
      return;
    }

    setStatus("loading", "Chargement du modèle 3D...");
    clearCurrentModel();

    try {
      const url = toModelUrl(source);
      const extension = fileExtension(source);

      if (extension === "glb" || extension === "gltf") {
        const [{ GLTFLoader }] = await Promise.all([
          import("three/examples/jsm/loaders/GLTFLoader.js"),
        ]);

        const loader = new GLTFLoader();
        const gltf = await loader.loadAsync(url);
        if (disposed) return;
        rootModel = gltf.scene;
      } else if (extension === "obj") {
        const [{ OBJLoader }] = await Promise.all([
          import("three/examples/jsm/loaders/OBJLoader.js"),
        ]);

        const loader = new OBJLoader();
        rootModel = await loader.loadAsync(url);
        if (disposed) return;
      } else {
        throw new Error(`Format de modele non pris en charge: ${extension || "inconnu"}`);
      }

      if (!rootModel) {
        throw new Error("Aucun objet 3D charge.");
      }

      scene.add(rootModel);
      centerCameraOnObject(rootModel);
      setStatus("idle", "");
    } catch (error) {
      console.error("3D viewer load error:", error);
      setStatus("error", "Impossible de charger le modèle 3D pour ce format.");
    }
  }

  function updateRendererSize() {
    if (!mountEl || !renderer || !camera) return;
    const width = mountEl.clientWidth || 320;
    const height = Math.max(mountEl.clientHeight || 280, 200);
    renderer.setSize(width, height);
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
  }

  function animate() {
    if (disposed || !renderer || !scene || !camera) return;
    controls?.update();
    renderer.render(scene, camera);
    frameId = requestAnimationFrame(animate);
  }

  function refreshFromProps() {
    if (!mounted || !scene) return;
    if (noVisualAssets) {
      clearCurrentModel();
      setStatus("empty", "Aucun fichier visuel disponible pour cette structure.");
      return;
    }

    const nextPath = modelPath ?? "";
    clearCurrentModel();
    if (nextPath.trim().length === 0) {
      setStatus("empty", "Aucun modèle 3D détecté.");
      return;
    }
    setStatus("empty", "Modèle prêt. Cliquez sur le bouton pour charger la visualisation 3D.");
  }

  async function loadCurrentModel() {
    if (status === "loading" || noVisualAssets) return;
    await loadModel(modelPath ?? "");
  }

  async function toggleViewerFullscreen() {
    isExpanded = !isExpanded;
    await tick();
    updateRendererSize();
  }
  
  const onResize = () => updateRendererSize();
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && isExpanded) {
      isExpanded = false;
      updateRendererSize();
    }
  };

  onMount(async () => {
    mounted = true;
    if (!mountEl) return;

    scene = new THREE.Scene();
    scene.background = new THREE.Color(0xf4f4f5);

    camera = new THREE.PerspectiveCamera(45, 1, 0.1, 5000);
    camera.position.set(3, 2, 4);

    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    mountEl.appendChild(renderer.domElement);

    const ambient = new THREE.AmbientLight(0xffffff, 0.7);
    scene.add(ambient);
    const key = new THREE.DirectionalLight(0xffffff, 0.9);
    key.position.set(4, 8, 6);
    scene.add(key);
    const fill = new THREE.DirectionalLight(0xffffff, 0.35);
    fill.position.set(-4, 3, -2);
    scene.add(fill);

    const { OrbitControls } = await import("three/examples/jsm/controls/OrbitControls.js");
    controls = new OrbitControls(camera, renderer.domElement);
    (controls as any).enableDamping = true;
    (controls as any).dampingFactor = 0.08;
    (controls as any).minDistance = 0.1;
    (controls as any).maxDistance = 10000;

    window.addEventListener("resize", onResize);
    window.addEventListener("keydown", onKeyDown);
    updateRendererSize();
    frameId = requestAnimationFrame(animate);
    refreshFromProps();
  });

  $: inputSignature = `${modelPath ?? ""}::${noVisualAssets ? "1" : "0"}`;
  $: if (mounted && inputSignature !== lastInputSignature) {
    lastInputSignature = inputSignature;
    refreshFromProps();
  }

  onDestroy(() => {
    disposed = true;
    mounted = false;
    window.removeEventListener("resize", onResize);
    window.removeEventListener("keydown", onKeyDown);
    if (frameId) cancelAnimationFrame(frameId);
    clearCurrentModel();
    controls?.dispose();
    renderer?.dispose();
    if (renderer?.domElement?.parentNode) {
      renderer.domElement.parentNode.removeChild(renderer.domElement);
    }
    controls = null;
    camera = null;
    scene = null;
    renderer = null;
  });
</script>

<section class="viewer-shell" bind:this={viewerShellEl} class:is-expanded={isExpanded}>
  <div class="canvas-wrap" bind:this={mountEl}></div>
  {#if !noVisualAssets && (modelPath ?? "").trim().length > 0}
    <div class="viewer-actions">
      <button
        type="button"
        class="viewer-icon-btn"
        on:click={toggleViewerFullscreen}
        aria-label={isExpanded ? "Réduire la vue 3D" : "Agrandir la vue 3D"}
        title={isExpanded ? "Réduire" : "Agrandir"}
      >
        {#if isExpanded}
          <svg viewBox="0 0 24 24" fill="none" width="14" height="14" aria-hidden="true">
            <path d="M6 6L18 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
            <path d="M18 6L6 18" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
          </svg>
        {:else}
          <svg viewBox="0 0 24 24" fill="none" width="14" height="14" aria-hidden="true">
            <path d="M14 10L20 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
            <path d="M20 9V4H15" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M10 14L4 20" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
            <path d="M4 15V20H9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        {/if}
      </button>
    </div>
  {/if}
  {#if status !== "idle"}
    <div class="viewer-overlay" class:is-error={status === "error"}>
      <p>{statusMessage}</p>
      {#if status === "empty" && !noVisualAssets && (modelPath ?? "").trim().length > 0}
        <button class="btn btn-secondary btn-sm overlay-load-btn" type="button" on:click={loadCurrentModel}>
          Charger le modèle 3D
        </button>
      {/if}
    </div>
  {/if}
</section>

<style>
  .viewer-shell {
    position: relative;
    border: 1px solid var(--color-neutral-300, #d6d6d6);
    border-radius: 10px;
    overflow: hidden;
    background: var(--color-neutral-100, #fafafa);
    min-height: 280px;
  }

  .canvas-wrap {
    width: 100%;
    height: 100%;
    min-height: 280px;
  }

  .canvas-wrap :global(canvas) {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }

  .viewer-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 12px;
    text-align: center;
    background: rgba(250, 250, 250, 0.88);
    color: var(--color-neutral-700, #4b5563);
    pointer-events: none;
  }

  .overlay-load-btn {
    pointer-events: auto;
  }

  .viewer-actions {
    position: absolute;
    top: 10px;
    right: 10px;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .viewer-icon-btn {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    border: 1px solid var(--color-neutral-400, #cbd5e1);
    background: rgba(255, 255, 255, 0.92);
    color: var(--color-neutral-800, #1f2937);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }

  .viewer-icon-btn:hover {
    background: #fff;
    border-color: var(--color-neutral-500, #94a3b8);
  }

  .viewer-icon-btn:focus-visible {
    outline: 2px solid rgba(37, 74, 125, 0.35);
    outline-offset: 1px;
  }

  .viewer-shell.is-expanded {
    position: fixed;
    inset: 0;
    z-index: 3000;
    border-radius: 0;
    min-height: 100dvh;
    background: #fff;
  }

  .viewer-shell.is-expanded .canvas-wrap {
    min-height: 100dvh;
  }

  .viewer-overlay.is-error {
    color: #8b1a1a;
    background: rgba(255, 243, 243, 0.92);
  }

  .viewer-overlay p {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }
</style>
