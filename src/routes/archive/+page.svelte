<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { replaceState } from "$app/navigation";
  import { afterUpdate, onDestroy, onMount } from "svelte";
  import ArchiveMetadataPanel from "$lib/components/archive/ArchiveMetadataPanel.svelte";
  import Archive3DViewer from "$lib/components/archive/Archive3DViewer.svelte";

  // --- Types ---
  interface SummaryItem {
    operation_code: string;
    operation_site: string;
    operation_type: string;
    responsable: string;
    structure_id: string;
    structure_type: string;
    model_author: string;
    path: string;
    has_model: boolean;
    has_orthos: boolean;
    has_production: boolean;
    last_modified_unix: number | null;
  }

  interface SummaryResponse {
    total: number;
    page: number;
    per_page: number;
    items: SummaryItem[];
  }

  interface ArchiveStatistics {
    total_structures: number;
    total_polygons: number;
    total_photos: number;
    total_size_bytes: number;
    total_models_size_bytes: number;
    total_photos_size_bytes: number;
    by_structure_type: Record<string, number>;
    by_operation: Record<string, number>;
    by_author: Record<string, number>;
    by_year: Record<string, number>;
  }

  interface InventoryItem {
    structure_id: string;
    operation_code: string;
    operation_site: string;
    operation_type: string;
    operation_responsable: string;
    structure_type: string;
    photos_count: string;
    photos_total_size_mb: string;
    model_size_mb: string;
    faces_count: string;
    model_author: string;
    depositor: string;
    software: string;
    deposit_date: string;
  }

  interface AuditEntry {
    timestamp: string;
    action: string;
    user: string;
    structure_path: string;
    metadata: unknown | null;
    success: boolean;
    error: string | null;
  }

  interface StructureDetailsResponse {
    metadata: unknown | null;
    metadata_raw: unknown | null;
    models: string[];
    orthos: string[];
    photos: string[];
    work_files: string[];
  }

  interface PresetOperation {
    code: string;
  }

  interface MetadataPresets {
    operations: PresetOperation[];
    structure_types: string[];
  }

  type ArchiveTab = "consultation" | "stats" | "inventory";
  type ConsultationAssetFilter = "all" | "3d" | "ortho" | "prod";
  type InventorySortField =
    | "structure_id"
    | "operation_code"
    | "operation_site"
    | "operation_type"
    | "operation_responsable"
    | "structure_type"
    | "photos_count"
    | "photos_total_size_mb"
    | "model_size_mb"
    | "faces_count"
    | "model_author"
    | "depositor"
    | "software"
    | "deposit_date";
  type InventoryFilterKey =
    | "operation_code"
    | "operation_site"
    | "operation_type"
    | "operation_responsable"
    | "structure_type"
    | "model_author"
    | "depositor"
    | "software";

  // --- Sub-tabs ---
  let activeTab: ArchiveTab = "consultation";

  // --- Consultation state ---
  let items: SummaryItem[] = [];
  let total = 0;
  let page = 1;
  let perPage = 50;
  let query = "";
  let loading = false;
  let consultationError: string | null = null;

  // --- Filters ---
  let filterAsset: ConsultationAssetFilter = "all";
  let filterOperationCode = "";
  let filterStructureType = "";
  let filterDateOrder: "" | "recent" | "old" = "";
  let operationFilterOptions: string[] = [];
  let structureTypeFilterOptions: string[] = [];

  // --- Sort ---
  let sortField = "operation_code";
  let sortDirection: "asc" | "desc" = "asc";

  // --- Detail ---
  let selectedItem: SummaryItem | null = null;
  let selectedDetails: StructureDetailsResponse | null = null;
  let detailLoading = false;
  let detailError: string | null = null;
  let detailActionStatus: string | null = null;
  let detailRequestToken = 0;
  let orthoPreviewSrc: string | null = null;
  let orthoPreviewPath: string | null = null;
  let orthoPreviewLoading = false;
  let orthoPreviewError: string | null = null;
  let showOrthoPreviewModal = false;

  // --- Stats ---
  let stats: ArchiveStatistics | null = null;
  let loadingStats = false;
  let statsError: string | null = null;

  // --- Activity ---
  let recentActivity: AuditEntry[] = [];
  let loadingActivity = false;
  let activityError: string | null = null;

  // --- Inventory ---
  let inventory: InventoryItem[] = [];
  let loadingInventory = false;
  let inventoryError: string | null = null;
  let inventoryQuery = "";
  let inventorySortField: InventorySortField = "deposit_date";
  let inventorySortDirection: "asc" | "desc" = "desc";
  let inventoryFilterOperationCode = "";
  let inventoryFilterOperationSite = "";
  let inventoryFilterOperationType = "";
  let inventoryFilterResponsable = "";
  let inventoryFilterStructureType = "";
  let inventoryFilterModelAuthor = "";
  let inventoryFilterDepositor = "";
  let inventoryFilterSoftware = "";
  let activeInventoryFilterMenu: string | null = null;
  let inventoryFilterMenuPos: { top: number; left: number } = { top: 0, left: 0 };
  let inventoryPage = 1;
  let inventoryPerPage = 50;
  let inventoryExportStatus = "";
  let inventoryExportStatusType: "success" | "error" | "" = "";
  let inventoryTableContainerEl: HTMLDivElement | null = null;
  let canScrollInventoryLeft = false;
  let canScrollInventoryRight = false;

  let consultationDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  let inventoryScrollRaf: number | null = null;
  let inventoryExportStatusTimer: ReturnType<typeof setTimeout> | null = null;

  // --- Stats interaction ---
  const statsTopLimit = 8;
  let showAllOperations = false;
  let showAllStructureTypes = false;
  let showAllAuthors = false;
  let showAllYears = false;

  // --- Derived ---
  $: totalPages = Math.ceil(total / perPage) || 1;

  // --- Mount ---
  onMount(() => {
    hydrateStateFromUrl();
    loadConsultationFilterOptions();
    loadArchive();
    if (activeTab === "stats") {
      loadStats();
      loadRecentActivity();
    }
    if (activeTab === "inventory") {
      loadInventory();
    }

    const onResize = () => scheduleInventoryScrollStateUpdate();
    window.addEventListener("resize", onResize);
    scheduleInventoryScrollStateUpdate();

    return () => {
      window.removeEventListener("resize", onResize);
    };
  });

  onDestroy(() => {
    if (inventoryScrollRaf !== null) cancelAnimationFrame(inventoryScrollRaf);
    if (inventoryExportStatusTimer !== null) clearTimeout(inventoryExportStatusTimer);
  });

  function showInventoryExportStatus(
    message: string,
    type: "success" | "error",
  ) {
    inventoryExportStatus = message;
    inventoryExportStatusType = type;
    if (inventoryExportStatusTimer !== null) {
      clearTimeout(inventoryExportStatusTimer);
    }
    inventoryExportStatusTimer = setTimeout(() => {
      inventoryExportStatus = "";
      inventoryExportStatusType = "";
      inventoryExportStatusTimer = null;
    }, 3200);
  }

  function parsePositiveInteger(raw: string | null, fallback: number): number {
    if (!raw) return fallback;
    const value = Number.parseInt(raw, 10);
    return Number.isFinite(value) && value > 0 ? value : fallback;
  }

  function hydrateStateFromUrl() {
    if (typeof window === "undefined") return;
    const params = new URLSearchParams(window.location.search);
    const tab = params.get("tab");
    if (tab === "consultation" || tab === "stats" || tab === "inventory") {
      activeTab = tab;
    }

    query = params.get("q") ?? "";
    const assetFilterParam = params.get("asset");
    if (
      assetFilterParam === "all" ||
      assetFilterParam === "3d" ||
      assetFilterParam === "ortho" ||
      assetFilterParam === "prod"
    ) {
      filterAsset = assetFilterParam;
    }
    filterOperationCode = params.get("opf") ?? "";
    filterStructureType = params.get("stf") ?? "";
    const dateOrder = params.get("date_order");
    if (dateOrder === "recent" || dateOrder === "old") {
      filterDateOrder = dateOrder;
    } else {
      filterDateOrder = "";
    }

    const sort = params.get("sort");
    if (sort === "operation_code" || sort === "structure_id" || sort === "modified_at") {
      sortField = sort;
    }

    const direction = params.get("dir");
    if (direction === "asc" || direction === "desc") {
      sortDirection = direction;
    }

    page = parsePositiveInteger(params.get("page"), 1);
    const per = parsePositiveInteger(params.get("per"), 50);
    perPage = [25, 50, 100].includes(per) ? per : 50;

    inventoryQuery = params.get("ivq") ?? "";
    inventoryFilterOperationCode = params.get("iv_op") ?? "";
    inventoryFilterOperationSite = params.get("iv_site") ?? "";
    inventoryFilterOperationType = params.get("iv_optype") ?? "";
    inventoryFilterResponsable = params.get("iv_resp") ?? "";
    inventoryFilterStructureType = params.get("iv_type") ?? "";
    inventoryFilterModelAuthor = params.get("iv_author") ?? "";
    inventoryFilterDepositor = params.get("iv_depositor") ?? "";
    inventoryFilterSoftware = params.get("iv_software") ?? "";
    inventoryPage = parsePositiveInteger(params.get("iv_page"), 1);
    const ivPer = parsePositiveInteger(params.get("iv_per"), 50);
    inventoryPerPage = [25, 50, 100].includes(ivPer) ? ivPer : 50;
  }

  function syncUrlState() {
    if (typeof window === "undefined") return;
    const params = new URLSearchParams();

    if (activeTab !== "consultation") params.set("tab", activeTab);
    if (query.trim()) params.set("q", query.trim());
    if (filterAsset !== "all") params.set("asset", filterAsset);
    if (filterOperationCode.trim()) params.set("opf", filterOperationCode.trim());
    if (filterStructureType.trim()) params.set("stf", filterStructureType.trim());
    if (filterDateOrder) params.set("date_order", filterDateOrder);
    if (sortField !== "operation_code") params.set("sort", sortField);
    if (sortDirection !== "asc") params.set("dir", sortDirection);
    if (page !== 1) params.set("page", String(page));
    if (perPage !== 50) params.set("per", String(perPage));
    if (inventoryQuery.trim()) params.set("ivq", inventoryQuery.trim());
    if (inventoryFilterOperationCode.trim()) params.set("iv_op", inventoryFilterOperationCode.trim());
    if (inventoryFilterOperationSite.trim()) params.set("iv_site", inventoryFilterOperationSite.trim());
    if (inventoryFilterOperationType.trim()) params.set("iv_optype", inventoryFilterOperationType.trim());
    if (inventoryFilterResponsable.trim()) params.set("iv_resp", inventoryFilterResponsable.trim());
    if (inventoryFilterStructureType.trim()) params.set("iv_type", inventoryFilterStructureType.trim());
    if (inventoryFilterModelAuthor.trim()) params.set("iv_author", inventoryFilterModelAuthor.trim());
    if (inventoryFilterDepositor.trim()) params.set("iv_depositor", inventoryFilterDepositor.trim());
    if (inventoryFilterSoftware.trim()) params.set("iv_software", inventoryFilterSoftware.trim());
    if (inventoryPage !== 1) params.set("iv_page", String(inventoryPage));
    if (inventoryPerPage !== 50) params.set("iv_per", String(inventoryPerPage));

    const queryString = params.toString();
    const nextUrl = `${window.location.pathname}${queryString ? `?${queryString}` : ""}`;
    replaceState(nextUrl, {});
  }

  // --- Consultation functions ---
  async function loadArchive() {
    consultationError = null;
    loading = true;
    syncUrlState();
    try {
      const filters: Record<string, unknown> = {};
      if (query.trim()) filters.query = query.trim();
      if (filterOperationCode.trim()) filters.operation_code = filterOperationCode.trim();
      if (filterStructureType.trim()) filters.structure_type = filterStructureType.trim();
      if (filterAsset === "3d") filters.has_model = true;
      if (filterAsset === "ortho") filters.has_orthos = true;
      if (filterAsset === "prod") filters.has_production = true;

      let effectiveSortField = sortField;
      let effectiveSortDirection = sortDirection;
      if (filterDateOrder === "recent") {
        effectiveSortField = "modified_at";
        effectiveSortDirection = "desc";
      } else if (filterDateOrder === "old") {
        effectiveSortField = "modified_at";
        effectiveSortDirection = "asc";
      }

      const res = await invoke<SummaryResponse>("get_structure_summary", {
        space: "Archive",
        filters: Object.keys(filters).length > 0 ? filters : null,
        pagination: { page, per_page: perPage },
        sort: { field: effectiveSortField, direction: effectiveSortDirection },
      });
      items = res.items;
      total = res.total;
      if (selectedItem) {
        const refreshedSelected =
          res.items.find((item) => item.path === selectedItem?.path) ?? null;
        if (!refreshedSelected) {
          clearSelectionDetails();
        } else {
          selectedItem = refreshedSelected;
        }
      }
    } catch (e) {
      console.error("Erreur chargement archive:", e);
      consultationError =
        "Impossible de charger les archives. Vérifiez la configuration et réessayez.";
    } finally {
      loading = false;
    }
  }

  async function loadConsultationFilterOptions() {
    try {
      const presets = await invoke<MetadataPresets>("get_metadata_presets");
      operationFilterOptions = [...new Set((presets.operations ?? []).map((op) => op.code).filter(Boolean))]
        .sort((a, b) => a.localeCompare(b, "fr"));
      structureTypeFilterOptions = [...new Set((presets.structure_types ?? []).filter(Boolean))]
        .sort((a, b) => a.localeCompare(b, "fr"));
    } catch (error) {
      console.warn("Impossible de charger les options de filtres archive:", error);
      operationFilterOptions = [];
      structureTypeFilterOptions = [];
    }
  }

  function handleConsultationTextInput() {
    page = 1;
    if (consultationDebounceTimer) {
      clearTimeout(consultationDebounceTimer);
    }
    consultationDebounceTimer = setTimeout(() => {
      loadArchive();
    }, 220);
  }

  function handleFilterChange() {
    if (consultationDebounceTimer) {
      clearTimeout(consultationDebounceTimer);
      consultationDebounceTimer = null;
    }
    page = 1;
    loadArchive();
  }

  function resetFilters() {
    query = "";
    filterAsset = "all";
    filterOperationCode = "";
    filterStructureType = "";
    filterDateOrder = "";
    sortField = "operation_code";
    sortDirection = "asc";
    page = 1;
    loadArchive();
  }

  function resetInventoryFilters() {
    inventoryQuery = "";
    inventoryFilterOperationCode = "";
    inventoryFilterOperationSite = "";
    inventoryFilterOperationType = "";
    inventoryFilterResponsable = "";
    inventoryFilterStructureType = "";
    inventoryFilterModelAuthor = "";
    inventoryFilterDepositor = "";
    inventoryFilterSoftware = "";
    activeInventoryFilterMenu = null;
    inventoryPage = 1;
    syncUrlState();
  }

  function formatNumber(n: number): string {
    return n.toLocaleString("fr-FR");
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return "0 o";
    const units = ["o", "Ko", "Mo", "Go", "To"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    const val = bytes / Math.pow(1024, i);
    return `${val < 10 ? val.toFixed(2) : val < 100 ? val.toFixed(1) : Math.round(val)} ${units[i]}`;
  }

  function stringifyError(error: unknown, fallback: string): string {
    if (typeof error === "string") return error;
    if (error instanceof Error) return error.message;
    return fallback;
  }

  function filename(path: string): string {
    const normalized = path.replace(/\\/g, "/");
    const parts = normalized.split("/");
    return parts[parts.length - 1] || path;
  }

  function preferredModelPath(paths: string[] | null | undefined): string | null {
    if (!paths || paths.length === 0) return null;

    const priority = ["glb", "gltf", "obj"];
    const byExtension = new Map<string, string[]>();

    for (const path of paths) {
      const name = filename(path);
      const dotIndex = name.lastIndexOf(".");
      const extension = dotIndex >= 0 ? name.slice(dotIndex + 1).toLowerCase() : "";
      if (!byExtension.has(extension)) byExtension.set(extension, []);
      byExtension.get(extension)?.push(path);
    }

    for (const extension of priority) {
      const match = byExtension.get(extension)?.[0];
      if (match) return match;
    }

    return null;
  }

  function clearSelectionDetails() {
    selectedItem = null;
    selectedDetails = null;
    detailLoading = false;
    detailError = null;
    detailActionStatus = null;
    detailRequestToken++;
    orthoPreviewSrc = null;
    orthoPreviewPath = null;
    orthoPreviewLoading = false;
    orthoPreviewError = null;
    showOrthoPreviewModal = false;
  }

  async function loadOrthoPreview(paths: string[], token: number) {
    orthoPreviewLoading = true;
    orthoPreviewError = null;
    orthoPreviewSrc = null;
    orthoPreviewPath = null;

    try {
      if (token !== detailRequestToken) return;
      let lastError: unknown = null;
      for (const path of paths) {
        try {
          orthoPreviewPath = path;
          orthoPreviewSrc = await invoke<string>("get_image_preview_data_url", {
            imagePath: path,
            maxDimension: 1600,
          });
          return;
        } catch (error) {
          lastError = error;
        }
      }

      throw lastError ?? new Error("Aucune orthophoto exploitable.");
    } catch (error) {
      if (token !== detailRequestToken) return;
      orthoPreviewError = stringifyError(
        error,
        "Impossible de charger la prévisualisation orthophoto.",
      );
    } finally {
      if (token !== detailRequestToken) return;
      orthoPreviewLoading = false;
    }
  }

  async function selectConsultationItem(item: SummaryItem) {
    selectedItem = item;
    selectedDetails = null;
    detailError = null;
    detailLoading = true;
    detailActionStatus = null;
    showOrthoPreviewModal = false;
    orthoPreviewSrc = null;
    orthoPreviewPath = null;
    orthoPreviewLoading = false;
    orthoPreviewError = null;

    const token = ++detailRequestToken;
    try {
      const details = await invoke<StructureDetailsResponse>("get_structure_details", {
        structurePath: item.path,
      });
      if (token !== detailRequestToken) return;

      selectedDetails = details;
      if (details.orthos.length > 0) {
        await loadOrthoPreview(details.orthos, token);
      } else {
        orthoPreviewError = "Aucune orthophoto disponible sur cette structure.";
      }
    } catch (error) {
      if (token !== detailRequestToken) return;
      detailError = stringifyError(error, "Impossible de charger le détail de la structure.");
    } finally {
      if (token !== detailRequestToken) return;
      detailLoading = false;
    }
  }

  async function openSelectedPath() {
    if (!selectedItem) return;
    detailActionStatus = null;
    try {
      await invoke("reveal_in_explorer", { path: selectedItem.path });
      detailActionStatus = "Emplacement ouvert dans l'explorateur.";
    } catch (error) {
      detailActionStatus = `Erreur: ${stringifyError(
        error,
        "Impossible d'ouvrir l'emplacement.",
      )}`;
    }
  }

  function asRecord(value: unknown): Record<string, unknown> | null {
    if (!value || typeof value !== "object" || Array.isArray(value)) return null;
    return value as Record<string, unknown>;
  }

  function valueToText(value: unknown): string {
    if (value === null || value === undefined) return "—";
    if (typeof value === "string") return value.trim() || "—";
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    if (Array.isArray(value)) return value.length ? value.map((item) => valueToText(item)).join(", ") : "—";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  }

  // --- Stats functions ---
  async function loadStats() {
    if (stats) return;
    statsError = null;
    loadingStats = true;
    syncUrlState();
    try {
      stats = await invoke<ArchiveStatistics>("get_archive_statistics");
    } catch (e) {
      console.error("Erreur chargement stats:", e);
      statsError =
        "Impossible de charger les statistiques de l'archive. Réessayez.";
    } finally {
      loadingStats = false;
    }
  }

  async function loadRecentActivity() {
    if (recentActivity.length > 0) return;
    activityError = null;
    loadingActivity = true;
    syncUrlState();
    try {
      recentActivity = await invoke<AuditEntry[]>("get_recent_validations", {
        limit: 10,
      });
    } catch (e) {
      console.error("Erreur chargement activite recente:", e);
      activityError = "Impossible de charger l'activité récente.";
    } finally {
      loadingActivity = false;
    }
  }

  // --- Inventory functions ---
  async function loadInventory() {
    if (inventory.length > 0) return;
    inventoryError = null;
    loadingInventory = true;
    syncUrlState();
    try {
      inventory = await invoke<InventoryItem[]>("generate_inventory");
    } catch (e) {
      console.error("Erreur inventaire:", e);
      inventoryError = "Impossible de générer l'inventaire. Réessayez.";
    } finally {
      loadingInventory = false;
    }
  }

  function exportInventoryCSV() {
    if (displayedInventory.length === 0) {
      showInventoryExportStatus("Aucune donnée à exporter.", "error");
      return;
    }

    try {
      const headers = [
        "Structure",
        "Opération",
        "Site",
        "Type opération",
        "Responsable",
        "Type",
        "Photos",
        "Taille photos (Mo)",
        "Taille modèle (Mo)",
        "Polygones",
        "Auteur modèle",
        "Déposant",
        "Logiciel",
        "Date dépôt",
      ];
      const rows = displayedInventory.map((i) => [
        i.structure_id,
        i.operation_code,
        i.operation_site,
        i.operation_type,
        i.operation_responsable,
        i.structure_type,
        i.photos_count,
        i.photos_total_size_mb,
        i.model_size_mb,
        i.faces_count,
        i.model_author,
        i.depositor,
        i.software,
        i.deposit_date,
      ]);

      const csvEscape = (value: unknown): string =>
        `"${String(value ?? "").replaceAll('"', '""')}"`;
      const csv = [headers, ...rows]
        .map((r) => r.map((c) => csvEscape(c)).join(";"))
        .join("\r\n");
      const blob = new Blob(["\uFEFF", csv], { type: "text/csv;charset=utf-8;" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "inventaire_archive.csv";
      a.click();
      URL.revokeObjectURL(url);
      showInventoryExportStatus("Export CSV terminé : inventaire_archive.csv", "success");
    } catch (e) {
      console.error("Erreur export CSV inventaire:", e);
      showInventoryExportStatus("Échec de l'export CSV. Réessayez.", "error");
    }
  }

  // --- Tab switching ---
  function switchTab(tab: ArchiveTab) {
    if (consultationDebounceTimer) {
      clearTimeout(consultationDebounceTimer);
      consultationDebounceTimer = null;
    }
    activeTab = tab;
    syncUrlState();
    if (tab === "stats") {
      loadStats();
      loadRecentActivity();
    }
    if (tab === "inventory") loadInventory();
  }

  function openConsultationFromStats(
    _kind: "operation" | "structure" | "author" | "year",
    label: string,
  ) {
    activeTab = "consultation";
    filterAsset = "all";
    query = label;

    page = 1;
    syncUrlState();
    loadArchive();
  }

  function toNumberOrNull(raw: string): number | null {
    const normalized = raw.replace(/\s/g, "").replace(",", ".");
    const value = Number.parseFloat(normalized);
    return Number.isFinite(value) ? value : null;
  }

  function normalizeText(value: string | null | undefined): string {
    return (value ?? "").toLowerCase().trim();
  }

  function compareInventoryItems(
    a: InventoryItem,
    b: InventoryItem,
    field: InventorySortField,
    direction: "asc" | "desc",
  ): number {
    const multiplier = direction === "asc" ? 1 : -1;
    if (
      field === "photos_count" ||
      field === "photos_total_size_mb" ||
      field === "model_size_mb" ||
      field === "faces_count"
    ) {
      const aNum = toNumberOrNull(a[field]) ?? -1;
      const bNum = toNumberOrNull(b[field]) ?? -1;
      if (aNum === bNum) return 0;
      return aNum > bNum ? multiplier : -multiplier;
    }

    const aVal = normalizeText(a[field]);
    const bVal = normalizeText(b[field]);
    if (aVal === bVal) return 0;
    return aVal > bVal ? multiplier : -multiplier;
  }

  function toggleInventorySort(field: InventorySortField) {
    if (inventorySortField === field) {
      inventorySortDirection = inventorySortDirection === "asc" ? "desc" : "asc";
    } else {
      inventorySortField = field;
      inventorySortDirection = "desc";
    }
    inventoryPage = 1;
    syncUrlState();
  }

  function inventorySortIcon(field: InventorySortField): string {
    if (inventorySortField !== field) return "";
    return inventorySortDirection === "asc" ? " ↑" : " ↓";
  }

  function isInventorySortActive(field: InventorySortField): boolean {
    return inventorySortField === field;
  }

  function isInventorySortDescending(field: InventorySortField): boolean {
    return inventorySortField === field && inventorySortDirection === "desc";
  }

  function toggleInventoryFilterMenu(key: string, event?: MouseEvent) {
    if (activeInventoryFilterMenu === key) {
      activeInventoryFilterMenu = null;
    } else {
      activeInventoryFilterMenu = key;
      if (event) {
        const btn = event.currentTarget as HTMLElement;
        const rect = btn.getBoundingClientRect();
        inventoryFilterMenuPos = { top: rect.bottom + 6, left: rect.right };
      }
    }
  }

  function setInventoryFilterValue(key: InventoryFilterKey, value: string) {
    const currentValue =
      key === "operation_code"
        ? inventoryFilterOperationCode
        : key === "operation_site"
          ? inventoryFilterOperationSite
          : key === "operation_type"
            ? inventoryFilterOperationType
            : key === "operation_responsable"
              ? inventoryFilterResponsable
              : key === "structure_type"
                ? inventoryFilterStructureType
                : key === "model_author"
                  ? inventoryFilterModelAuthor
                  : key === "depositor"
                    ? inventoryFilterDepositor
                    : inventoryFilterSoftware;

    const nextValue = currentValue === value ? "" : value;

    if (key === "operation_code") inventoryFilterOperationCode = nextValue;
    if (key === "operation_site") inventoryFilterOperationSite = nextValue;
    if (key === "operation_type") inventoryFilterOperationType = nextValue;
    if (key === "operation_responsable") inventoryFilterResponsable = nextValue;
    if (key === "structure_type") inventoryFilterStructureType = nextValue;
    if (key === "model_author") inventoryFilterModelAuthor = nextValue;
    if (key === "depositor") inventoryFilterDepositor = nextValue;
    if (key === "software") inventoryFilterSoftware = nextValue;

    inventoryPage = 1;
    activeInventoryFilterMenu = null;
    syncUrlState();
  }

  function updateInventoryScrollState() {
    const el = inventoryTableContainerEl;
    if (!el) {
      canScrollInventoryLeft = false;
      canScrollInventoryRight = false;
      return;
    }

    const maxScroll = Math.max(0, el.scrollWidth - el.clientWidth);
    canScrollInventoryLeft = el.scrollLeft > 4;
    canScrollInventoryRight = maxScroll - el.scrollLeft > 4;
  }

  function scheduleInventoryScrollStateUpdate() {
    if (inventoryScrollRaf !== null) cancelAnimationFrame(inventoryScrollRaf);
    inventoryScrollRaf = requestAnimationFrame(() => {
      inventoryScrollRaf = null;
      updateInventoryScrollState();
    });
  }

  function scrollInventoryHorizontally(direction: "left" | "right") {
    if (!inventoryTableContainerEl) return;
    const step = Math.max(260, Math.floor(inventoryTableContainerEl.clientWidth * 0.72));
    const delta = direction === "left" ? -step : step;
    inventoryTableContainerEl.scrollBy({ left: delta, behavior: "smooth" });
  }

  afterUpdate(() => {
    if (activeTab !== "inventory") return;
    scheduleInventoryScrollStateUpdate();
  });

  function uniqueSortedValues(values: string[]): string[] {
    const normalized = values
      .map((value) => (value ?? "").trim())
      .filter((value) => value.length > 0);
    return [...new Set(normalized)].sort((a, b) => a.localeCompare(b, "fr"));
  }

  // --- Bar chart helpers ---
  function maxValue(obj: Record<string, number>): number {
    const values = Object.values(obj);
    return values.length > 0 ? Math.max(...values) : 1;
  }

  function sortedEntries(obj: Record<string, number>): [string, number][] {
    return Object.entries(obj).sort((a, b) => b[1] - a[1]);
  }

  function structureName(path: string): string {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || parts[parts.length - 2] || path;
  }

  function formatTimestamp(ts: string): string {
    try {
      const d = new Date(ts);
      return `${d.toLocaleDateString("fr-FR")} ${d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })}`;
    } catch {
      return ts;
    }
  }

  $: metadataSource = selectedDetails?.metadata_raw ?? selectedDetails?.metadata ?? null;
  $: metadataRoot = asRecord(metadataSource);
  $: operationMetadata = asRecord(metadataRoot?.operation);
  $: structureMetadata = asRecord(metadataRoot?.structure);
  $: operationRows = [
    { label: "Code", value: valueToText(operationMetadata?.code) },
    { label: "Site", value: valueToText(operationMetadata?.site) },
    { label: "Responsable", value: valueToText(operationMetadata?.responsable) },
  ];
  $: structureRows = [
    { label: "Type d'opération", value: valueToText(operationMetadata?.op_type) },
    { label: "Identifiant structure", value: valueToText(structureMetadata?.id) },
    { label: "Type de structure", value: valueToText(structureMetadata?.st_type) },
    { label: "Auteur du modèle", value: valueToText(structureMetadata?.model_author) },
    { label: "Déposant", value: valueToText(structureMetadata?.depositor) },
    { label: "Nombre de photos", value: valueToText(structureMetadata?.photos_count) },
    { label: "Nombre de polygones", value: valueToText(structureMetadata?.faces_count) },
    { label: "Logiciels", value: valueToText(structureMetadata?.software) },
  ];
  $: structureDescription = valueToText(structureMetadata?.description);
  $: detailModelCount = selectedDetails?.models?.length ?? (selectedItem?.has_model ? 1 : 0);
  $: detailOrthoCount =
    selectedDetails?.orthos?.length ?? (selectedItem?.has_orthos ? 1 : 0);
  $: detailPrimaryModelPath = preferredModelPath(selectedDetails?.models);
  $: detailNoVisualAssets = detailModelCount === 0 && detailOrthoCount === 0;
  $: operationEntries = stats ? sortedEntries(stats.by_operation) : [];
  $: structureEntries = stats ? sortedEntries(stats.by_structure_type) : [];
  $: authorEntries = stats ? sortedEntries(stats.by_author) : [];
  $: yearEntries = stats ? sortedEntries(stats.by_year) : [];
  $: visibleOperationEntries = showAllOperations
    ? operationEntries
    : operationEntries.slice(0, statsTopLimit);
  $: visibleStructureEntries = showAllStructureTypes
    ? structureEntries
    : structureEntries.slice(0, statsTopLimit);
  $: visibleAuthorEntries = showAllAuthors
    ? authorEntries
    : authorEntries.slice(0, statsTopLimit);
  $: visibleYearEntries = showAllYears
    ? yearEntries
    : yearEntries.slice(0, statsTopLimit);

  $: inventoryQueryNormalized = normalizeText(inventoryQuery);
  $: inventoryFilterOperationCodeNorm = normalizeText(inventoryFilterOperationCode);
  $: inventoryFilterOperationSiteNorm = normalizeText(inventoryFilterOperationSite);
  $: inventoryFilterOperationTypeNorm = normalizeText(inventoryFilterOperationType);
  $: inventoryFilterResponsableNorm = normalizeText(inventoryFilterResponsable);
  $: inventoryFilterStructureTypeNorm = normalizeText(inventoryFilterStructureType);
  $: inventoryFilterModelAuthorNorm = normalizeText(inventoryFilterModelAuthor);
  $: inventoryFilterDepositorNorm = normalizeText(inventoryFilterDepositor);
  $: inventoryFilterSoftwareNorm = normalizeText(inventoryFilterSoftware);
  $: inventoryHasActiveFilters =
    !!inventoryQuery.trim() ||
    !!inventoryFilterOperationCode ||
    !!inventoryFilterOperationSite ||
    !!inventoryFilterOperationType ||
    !!inventoryFilterResponsable ||
    !!inventoryFilterStructureType ||
    !!inventoryFilterModelAuthor ||
    !!inventoryFilterDepositor ||
    !!inventoryFilterSoftware;
  $: inventoryOperationCodeOptions = uniqueSortedValues(inventory.map((item) => item.operation_code));
  $: inventoryOperationSiteOptions = uniqueSortedValues(inventory.map((item) => item.operation_site));
  $: inventoryOperationTypeOptions = uniqueSortedValues(inventory.map((item) => item.operation_type));
  $: inventoryResponsableOptions = uniqueSortedValues(inventory.map((item) => item.operation_responsable));
  $: inventoryStructureTypeOptions = uniqueSortedValues(inventory.map((item) => item.structure_type));
  $: inventoryAuthorOptions = uniqueSortedValues(inventory.map((item) => item.model_author));
  $: inventoryDepositorOptions = uniqueSortedValues(inventory.map((item) => item.depositor));
  $: inventorySoftwareOptions = uniqueSortedValues(inventory.map((item) => item.software));
  $: filteredInventory = inventory.filter((item) => {
    const haystack = `${item.structure_id} ${item.operation_code} ${item.operation_site} ${item.operation_type} ${item.operation_responsable} ${item.structure_type} ${item.photos_count} ${item.photos_total_size_mb} ${item.model_size_mb} ${item.faces_count} ${item.model_author} ${item.depositor} ${item.software} ${item.deposit_date}`
      .toLowerCase()
      .trim();
    if (inventoryQueryNormalized && !haystack.includes(inventoryQueryNormalized)) return false;
    if (
      inventoryFilterOperationCodeNorm &&
      normalizeText(item.operation_code) !== inventoryFilterOperationCodeNorm
    )
      return false;
    if (
      inventoryFilterOperationSiteNorm &&
      normalizeText(item.operation_site) !== inventoryFilterOperationSiteNorm
    )
      return false;
    if (
      inventoryFilterOperationTypeNorm &&
      normalizeText(item.operation_type) !== inventoryFilterOperationTypeNorm
    )
      return false;
    if (
      inventoryFilterResponsableNorm &&
      normalizeText(item.operation_responsable) !== inventoryFilterResponsableNorm
    )
      return false;
    if (
      inventoryFilterStructureTypeNorm &&
      normalizeText(item.structure_type) !== inventoryFilterStructureTypeNorm
    )
      return false;
    if (
      inventoryFilterModelAuthorNorm &&
      normalizeText(item.model_author) !== inventoryFilterModelAuthorNorm
    )
      return false;
    if (
      inventoryFilterDepositorNorm &&
      normalizeText(item.depositor) !== inventoryFilterDepositorNorm
    )
      return false;
    if (
      inventoryFilterSoftwareNorm &&
      normalizeText(item.software) !== inventoryFilterSoftwareNorm
    )
      return false;
    return true;
  });
  $: displayedInventory = [...filteredInventory].sort((a, b) =>
    compareInventoryItems(a, b, inventorySortField, inventorySortDirection),
  );
  $: inventoryTotalPages = Math.max(
    1,
    Math.ceil(displayedInventory.length / inventoryPerPage),
  );
  $: if (inventoryPage > inventoryTotalPages) {
    inventoryPage = inventoryTotalPages;
  }
  $: inventoryPageStart =
    displayedInventory.length === 0 ? 0 : (inventoryPage - 1) * inventoryPerPage + 1;
  $: inventoryPageEnd = Math.min(inventoryPage * inventoryPerPage, displayedInventory.length);
  $: paginatedInventory = displayedInventory.slice(
    (inventoryPage - 1) * inventoryPerPage,
    inventoryPage * inventoryPerPage,
  );
</script>

<svelte:window on:click={() => { activeInventoryFilterMenu = null; }} />

<main class="archive-page" class:archive-page-inventory={activeTab === "inventory"}>
  <!-- HEADER -->
  <div class="page-header">
    <div class="header-left">
      <h1>Archive</h1>
      <p class="header-subtitle">
        {total} structure{total !== 1 ? "s" : ""} archivée{total !== 1
          ? "s"
          : ""}
      </p>
    </div>
  </div>

  <!-- TABS -->
  <div class="tabs-bar">
    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === "consultation"}
        on:click={() => switchTab("consultation")}
      >
        <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
          ><path
            d="M21 21L16.65 16.65M19 11C19 15.4183 15.4183 19 11 19C6.58172 19 3 15.4183 3 11C3 6.58172 6.58172 3 11 3C15.4183 3 19 6.58172 19 11Z"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          /></svg
        >
        Consultation
      </button>
      <button
        class="tab"
        class:active={activeTab === "stats"}
        on:click={() => switchTab("stats")}
      >
        <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
          ><path
            d="M18 20V10M12 20V4M6 20V14"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          /></svg
        >
        Statistiques
      </button>
      <button
        class="tab"
        class:active={activeTab === "inventory"}
        on:click={() => switchTab("inventory")}
      >
        <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
          ><path
            d="M9 5H7C5.89543 5 5 5.89543 5 7V19C5 20.1046 5.89543 21 7 21H17C18.1046 21 19 20.1046 19 19V7C19 5.89543 18.1046 5 17 5H15M9 5C9 6.10457 9.89543 7 11 7H13C14.1046 7 15 6.10457 15 5M9 5C9 3.89543 9.89543 3 11 3H13C14.1046 3 15 3.89543 15 5M12 12H15M12 16H15M9 12H9.01M9 16H9.01"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
          /></svg
        >
        Inventaire
      </button>
    </div>
  </div>

  <!-- ========================== -->
  <!--   TAB : CONSULTATION       -->
  <!-- ========================== -->
  {#if activeTab === "consultation"}
    <div class="tab-content">
      <!-- Barre de recherche -->
      <div class="toolbar">
        <div class="search-row">
          <div class="search-field">
            <svg
              class="search-icon"
              viewBox="0 0 24 24"
              fill="none"
              width="16"
              height="16"
              ><path
                d="M21 21L16.65 16.65M19 11C19 15.4183 15.4183 19 11 19C6.58172 19 3 15.4183 3 11C3 6.58172 6.58172 3 11 3C15.4183 3 19 6.58172 19 11Z"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              /></svg
            >
            <input
              class="search-input"
              placeholder="Rechercher par code, site, structure, auteur..."
              bind:value={query}
              on:input={handleConsultationTextInput}
              on:keydown={(e) => e.key === "Enter" && handleFilterChange()}
            />
          </div>
          <button class="btn btn-primary search-submit-btn" on:click={handleFilterChange}>
            Rechercher
          </button>
        </div>

        <div class="consultation-toolbar">
          {#if query || filterAsset !== "all" || filterOperationCode || filterStructureType || filterDateOrder}
            <button class="btn-link" on:click={resetFilters}>Réinitialiser</button>
          {/if}
        </div>
      </div>

      <div
        class="consultation-layout"
        class:consultation-layout-empty={!selectedItem}
        class:consultation-layout-has-selection={!!selectedItem}
      >
        <section class="consultation-list-panel">
          <div class="consultation-extra-filters">
            <div class="consultation-filter-chip">
              <div class="meta-select-wrapper">
                <select
                  id="archive-filter-operation"
                  class="meta-select chip-select"
                  bind:value={filterOperationCode}
                  on:change={handleFilterChange}
                >
                  <option value="">Opération</option>
                  {#each operationFilterOptions as operationCode}
                    <option value={operationCode}>{operationCode}</option>
                  {/each}
                </select>
              </div>
            </div>

            <div class="consultation-filter-chip">
              <div class="meta-select-wrapper">
                <select
                  id="archive-filter-structure-type"
                  class="meta-select chip-select"
                  bind:value={filterStructureType}
                  on:change={handleFilterChange}
                >
                  <option value="">Type</option>
                  {#each structureTypeFilterOptions as structureType}
                    <option value={structureType}>{structureType}</option>
                  {/each}
                </select>
              </div>
            </div>

            <div class="consultation-filter-chip">
              <div class="meta-select-wrapper">
                <select
                  id="archive-filter-date-order"
                  class="meta-select chip-select"
                  bind:value={filterDateOrder}
                  on:change={handleFilterChange}
                >
                  <option value="">Date</option>
                  <option value="recent">Plus récent</option>
                  <option value="old">Plus ancien</option>
                </select>
              </div>
            </div>
          </div>

          {#if consultationError}
            <div class="state-banner state-banner-error">
              <span>{consultationError}</span>
              <button class="btn btn-secondary btn-sm" on:click={loadArchive}>
                Réessayer
              </button>
            </div>
          {/if}

          {#if loading}
            <div class="loading-state"><span class="loader"></span></div>
          {:else if items.length === 0}
            <div class="empty-state">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                width="40"
                height="40"
                stroke="currentColor"
                stroke-width="1.5"
              >
                <path
                  d="M20 7L12 3L4 7M20 7L12 11M20 7V17L12 21M12 11L4 7M12 11V21M4 7V17L12 21"
                />
              </svg>
              <p>Aucune structure trouvée dans les archives.</p>
            </div>
          {:else}
            <div class="consultation-list">
              {#each items as item}
                <button
                  type="button"
                  class="consultation-item"
                  class:selected={selectedItem?.path === item.path}
                  on:click={() => selectConsultationItem(item)}
                >
                  <div class="consultation-item-main">
                    <div class="consultation-item-structure">{item.structure_id}</div>
                    <div class="consultation-item-meta">
                      {item.operation_code} · {item.operation_site || "Site non renseigné"}
                    </div>
                  </div>
                  <div class="cell-badges">
                    {#if item.has_model}<span class="asset-badge asset-3d">3D</span>{/if}
                    {#if item.has_orthos}<span class="asset-badge asset-ortho">Ortho</span>{/if}
                    {#if item.has_production}<span class="asset-badge asset-prod">Prod</span>{/if}
                  </div>
                </button>
              {/each}
            </div>

            <div class="pagination consultation-pagination">
              <div class="pagination-controls">
                <button
                  class="btn btn-secondary btn-sm"
                  aria-label="Page précédente"
                  title="Page précédente"
                  disabled={page <= 1}
                  on:click={() => {
                    page--;
                    loadArchive();
                  }}
                >
                  <svg viewBox="0 0 24 24" fill="none" width="14" height="14"
                    ><path
                      d="M15 18L9 12L15 6"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    /></svg
                  >
                </button>
                <span class="page-num">Page {page} / {totalPages}</span>
                <button
                  class="btn btn-secondary btn-sm"
                  aria-label="Page suivante"
                  title="Page suivante"
                  disabled={page >= totalPages}
                  on:click={() => {
                    page++;
                    loadArchive();
                  }}
                >
                  <svg viewBox="0 0 24 24" fill="none" width="14" height="14"
                    ><path
                      d="M9 18L15 12L9 6"
                      stroke="currentColor"
                      stroke-width="2"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    /></svg
                  >
                </button>
              </div>
              <div class="page-size-control">
                <label for="archive-page-size">Par page</label>
                <select
                  id="archive-page-size"
                  bind:value={perPage}
                  on:change={() => {
                    page = 1;
                    loadArchive();
                  }}
                >
                  <option value={25}>25</option>
                  <option value={50}>50</option>
                  <option value={100}>100</option>
                </select>
              </div>
            </div>
          {/if}
        </section>

        <section class="consultation-detail-panel">
          {#if !selectedItem}
            <div class="consultation-detail-empty">
              <p>Sélectionnez une structure à gauche pour afficher son détail.</p>
            </div>
          {:else}
            <div class="detail-header">
              <div class="detail-title-wrap">
                <h2>{selectedItem.structure_id}</h2>
                <p>{selectedItem.operation_code} · {selectedItem.operation_site || "Site non renseigné"}</p>
                <div class="detail-assets-summary" aria-label="Informations assets">
                  <div class="cell-badges">
                    <span class="asset-badge asset-3d">3D</span>
                    <span class="asset-badge asset-ortho">Ortho</span>
                  </div>
                </div>
              </div>
              <div class="detail-header-right">
                <div class="detail-header-actions">
                  <button
                    class="btn btn-secondary detail-open-path-btn"
                    on:click={openSelectedPath}
                    aria-label="Aller à l'emplacement"
                    title="Aller à l'emplacement"
                  >
                    <svg viewBox="0 0 24 24" fill="none" width="15" height="15"
                      ><path
                        d="M18 13V19C18 20.1046 17.1046 21 16 21H5C3.89543 21 3 20.1046 3 19V8C3 6.89543 3.89543 6 5 6H11"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      /><path
                        d="M15 3H21V9"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      /><path
                        d="M10 14L21 3"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                      /></svg
                    >
                    <span>Aller à l'emplacement</span>
                  </button>
                  <button
                    class="btn btn-secondary detail-mobile-close-btn"
                    on:click={clearSelectionDetails}
                    aria-label="Fermer le détail"
                    title="Fermer"
                  >
                    ✕
                  </button>
                </div>
              </div>
            </div>

            {#if detailActionStatus}
              <p class="detail-action-status">{detailActionStatus}</p>
            {/if}

            {#if detailLoading}
              <div class="loading-state detail-loading"><span class="loader"></span></div>
            {:else if detailError}
              <div class="state-banner state-banner-error">
                <span>{detailError}</span>
                <button
                  class="btn btn-secondary btn-sm"
                  on:click={() => selectedItem && selectConsultationItem(selectedItem)}
                >
                  Réessayer
                </button>
              </div>
            {:else if selectedDetails}
              <div class="detail-viewers-grid" class:detail-viewers-grid-empty={detailNoVisualAssets}>
                <div class="detail-section detail-viewer-card" class:detail-viewer-card-compact={detailNoVisualAssets}>
                  <h3>Orthophotos</h3>
                  {#if detailNoVisualAssets}
                    <p class="detail-muted">Aucun fichier 3D ni image disponible.</p>
                  {:else if orthoPreviewLoading}
                    <p class="detail-muted">Chargement de la vignette...</p>
                  {:else if orthoPreviewSrc && orthoPreviewPath}
                    <div class="ortho-preview">
                      <div class="ortho-preview-thumb-wrap">
                        <button
                          type="button"
                          class="ortho-viewer-icon-btn"
                          on:click={() => (showOrthoPreviewModal = true)}
                          aria-label="Agrandir l'orthophoto"
                          title="Agrandir"
                        >
                          <svg viewBox="0 0 24 24" fill="none" width="14" height="14" aria-hidden="true">
                            <path d="M14 10L20 4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
                            <path d="M20 9V4H15" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
                            <path d="M10 14L4 20" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
                            <path d="M4 15V20H9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
                          </svg>
                        </button>
                        <button
                          type="button"
                          class="ortho-preview-thumb"
                          on:click={() => (showOrthoPreviewModal = true)}
                          aria-label="Agrandir l'orthophoto"
                        >
                          <img src={orthoPreviewSrc} alt={`Aperçu orthophoto ${selectedItem.structure_id}`} />
                        </button>
                      </div>
                      <div class="ortho-preview-caption">
                        <span>{filename(orthoPreviewPath)}</span>
                      </div>
                    </div>
                  {:else}
                    <p class="detail-muted">{orthoPreviewError || "Aucune orthophoto exploitable."}</p>
                  {/if}
                </div>

                <div class="detail-section detail-viewer-card" class:detail-viewer-card-compact={detailNoVisualAssets}>
                  <h3>Visualisation 3D</h3>
                  <Archive3DViewer modelPath={detailPrimaryModelPath} noVisualAssets={detailNoVisualAssets} />
                </div>
              </div>

              <div class="detail-section metadata-panel">
                <div class="metadata-panel-header">
                  <h3>Métadonnées</h3>
                  <p>Informations principales du dépôt, présentées pour une lecture rapide.</p>
                </div>
                <ArchiveMetadataPanel
                  operationItems={operationRows}
                  structureItems={structureRows}
                  description={structureDescription}
                />
                {#if !metadataSource}
                  <div class="detail-muted">
                    Aucune métadonnée disponible.
                  </div>
                {/if}
              </div>
            {/if}
          {/if}
        </section>
      </div>

      {#if selectedItem}
        <button
          class="consultation-mobile-overlay"
          aria-label="Fermer le détail"
          on:click={clearSelectionDetails}
        ></button>
      {/if}

      {#if showOrthoPreviewModal && orthoPreviewSrc}
        <div
          class="ortho-modal-backdrop"
          role="dialog"
          aria-modal="true"
          aria-label="Prévisualisation orthophoto"
          tabindex="0"
          on:click={() => (showOrthoPreviewModal = false)}
          on:keydown={(event) => event.key === "Escape" && (showOrthoPreviewModal = false)}
        >
          <button
            type="button"
            class="ortho-modal-close"
            on:click|stopPropagation={() => (showOrthoPreviewModal = false)}
            aria-label="Fermer l'aperçu orthophoto"
          >
            ×
          </button>
          <img
            src={orthoPreviewSrc}
            alt={`Orthophoto ${selectedItem?.structure_id ?? ""}`}
          />
        </div>
      {/if}
    </div>
  {/if}

  <!-- ========================== -->
  <!--   TAB : STATISTIQUES       -->
  <!-- ========================== -->
  {#if activeTab === "stats"}
    <div class="tab-content">
      {#if statsError}
        <div class="state-banner state-banner-error">
          <span>{statsError}</span>
          <button class="btn btn-secondary btn-sm" on:click={loadStats}>
            Réessayer
          </button>
        </div>
      {:else if loadingStats}
        <div class="loading-state"><span class="loader"></span></div>
      {:else if stats}
        <!-- Stat cards -->
        <div class="stat-cards">
          <div class="stat-card">
            <div class="stat-icon icon-structures">
              <!-- Monument / colonnes antiques -->
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M3 21H21" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M5 21V10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M8 21V10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M16 21V10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M19 21V10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M4 10H20" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M12 4L21 8H3L12 4Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
              </svg>
            </div>
            <div class="stat-body">
              <span class="stat-value">{formatNumber(stats.total_structures)}</span>
              <span class="stat-label">Structures archivées</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon icon-operations">
              <!-- Appareil photo -->
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M23 19C23 19.5304 22.7893 20.0391 22.4142 20.4142C22.0391 20.7893 21.5304 21 21 21H3C2.46957 21 1.96086 20.7893 1.58579 20.4142C1.21071 20.0391 1 19.5304 1 19V8C1 7.46957 1.21071 6.96086 1.58579 6.58579C1.96086 6.21071 2.46957 6 3 6H7L9 3H15L17 6H21C21.5304 6 22.0391 6.21071 22.4142 6.58579C22.7893 6.96086 23 7.46957 23 8V19Z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                <circle cx="12" cy="13" r="4" stroke="currentColor" stroke-width="1.5"/>
              </svg>
            </div>
            <div class="stat-body">
              <span class="stat-value">{formatNumber(stats.total_photos)}</span>
              <span class="stat-label">Photos</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon icon-size">
              <!-- Images empilées + poids -->
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <rect x="3" y="6" width="16" height="13" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <path d="M6 6V5C6 4.44772 6.44772 4 7 4H20C20.5523 4 21 4.44772 21 5V16C21 16.5523 20.5523 17 20 17H19" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <circle cx="8" cy="10" r="1.5" stroke="currentColor" stroke-width="1.3"/>
                <path d="M3 16L7 12L10 15L13 12L19 17" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div class="stat-body">
              <span class="stat-value">{formatSize(stats.total_photos_size_bytes)}</span>
              <span class="stat-label">Taille photos</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon icon-models">
              <!-- Cube 3D -->
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <path d="M12 3L21 8V16L12 21L3 16V8L12 3Z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round"/>
                <path d="M12 3V21" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                <path d="M3 8L12 13L21 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div class="stat-body">
              <span class="stat-value">{formatSize(stats.total_models_size_bytes)}</span>
              <span class="stat-label">Modèles 3D</span>
            </div>
          </div>
          <div class="stat-card">
            <div class="stat-icon icon-archive-size">
              <!-- Serveur / disque dur stack -->
              <svg viewBox="0 0 24 24" fill="none" width="22" height="22">
                <rect x="2" y="3" width="20" height="5" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <rect x="2" y="10" width="20" height="5" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <rect x="2" y="17" width="20" height="4" rx="1.5" stroke="currentColor" stroke-width="1.5"/>
                <circle cx="18" cy="5.5" r="1" fill="currentColor"/>
                <circle cx="18" cy="12.5" r="1" fill="currentColor"/>
                <circle cx="18" cy="19" r="1" fill="currentColor"/>
              </svg>
            </div>
            <div class="stat-body">
              <span class="stat-value">{formatSize(stats.total_size_bytes)}</span>
              <span class="stat-label">Taille dossier archive</span>
            </div>
          </div>
        </div>

        <!-- Repartitions -->
        <div class="breakdowns-grid">
          <!-- Par operation -->
          {#if operationEntries.length > 0}
            <div class="breakdown-card">
              <h3 class="breakdown-title">Par opération</h3>
              <div class="bar-list">
                {#each visibleOperationEntries as [label, count]}
                  <button
                    type="button"
                    class="bar-row bar-row-btn"
                    on:click={() => openConsultationFromStats("operation", label)}
                    title={`Filtrer la consultation sur l'opération "${label}"`}
                  >
                    <span class="bar-label" title={label}>{label}</span>
                    <div class="bar-track">
                      <div
                        class="bar-fill bar-primary"
                        style="width: {(count / maxValue(stats.by_operation)) *
                          100}%"
                      ></div>
                    </div>
                    <span class="bar-value">{count}</span>
                  </button>
                {/each}
              </div>
              {#if operationEntries.length > statsTopLimit}
                <div class="breakdown-actions">
                  <button
                    type="button"
                    class="btn-link"
                    on:click={() => (showAllOperations = !showAllOperations)}
                  >
                    {showAllOperations ? "Voir moins" : "Voir tout"}
                  </button>
                </div>
              {/if}
            </div>
          {/if}

          <!-- Par type -->
          {#if structureEntries.length > 0}
            <div class="breakdown-card">
              <h3 class="breakdown-title">Par type de structure</h3>
              <div class="bar-list">
                {#each visibleStructureEntries as [label, count]}
                  <button
                    type="button"
                    class="bar-row bar-row-btn"
                    on:click={() => openConsultationFromStats("structure", label)}
                    title={`Filtrer la consultation sur le type "${label}"`}
                  >
                    <span class="bar-label" title={label}>{label}</span>
                    <div class="bar-track">
                      <div
                        class="bar-fill bar-secondary"
                        style="width: {(count /
                          maxValue(stats.by_structure_type)) *
                          100}%"
                      ></div>
                    </div>
                    <span class="bar-value">{count}</span>
                  </button>
                {/each}
              </div>
              {#if structureEntries.length > statsTopLimit}
                <div class="breakdown-actions">
                  <button
                    type="button"
                    class="btn-link"
                    on:click={() => (showAllStructureTypes = !showAllStructureTypes)}
                  >
                    {showAllStructureTypes ? "Voir moins" : "Voir tout"}
                  </button>
                </div>
              {/if}
            </div>
          {/if}

          <!-- Par auteur -->
          {#if authorEntries.length > 0}
            <div class="breakdown-card">
              <h3 class="breakdown-title">Par auteur</h3>
              <div class="bar-list">
                {#each visibleAuthorEntries as [label, count]}
                  <button
                    type="button"
                    class="bar-row bar-row-btn"
                    on:click={() => openConsultationFromStats("author", label)}
                    title={`Filtrer la consultation sur l'auteur "${label}"`}
                  >
                    <span class="bar-label" title={label}>{label}</span>
                    <div class="bar-track">
                      <div
                        class="bar-fill bar-info"
                        style="width: {(count / maxValue(stats.by_author)) *
                          100}%"
                      ></div>
                    </div>
                    <span class="bar-value">{count}</span>
                  </button>
                {/each}
              </div>
              {#if authorEntries.length > statsTopLimit}
                <div class="breakdown-actions">
                  <button
                    type="button"
                    class="btn-link"
                    on:click={() => (showAllAuthors = !showAllAuthors)}
                  >
                    {showAllAuthors ? "Voir moins" : "Voir tout"}
                  </button>
                </div>
              {/if}
            </div>
          {/if}

          <!-- Par année -->
          {#if yearEntries.length > 0}
            <div class="breakdown-card">
              <h3 class="breakdown-title">Par année</h3>
              <div class="bar-list">
                {#each visibleYearEntries as [label, count]}
                  <button
                    type="button"
                    class="bar-row bar-row-btn"
                    on:click={() => openConsultationFromStats("year", label)}
                    title={`Rechercher l'année "${label}" dans la consultation`}
                  >
                    <span class="bar-label" title={label}>{label}</span>
                    <div class="bar-track">
                      <div
                        class="bar-fill bar-success"
                        style="width: {(count / maxValue(stats.by_year)) *
                          100}%"
                      ></div>
                    </div>
                    <span class="bar-value">{count}</span>
                  </button>
                {/each}
              </div>
              {#if yearEntries.length > statsTopLimit}
                <div class="breakdown-actions">
                  <button
                    type="button"
                    class="btn-link"
                    on:click={() => (showAllYears = !showAllYears)}
                  >
                    {showAllYears ? "Voir moins" : "Voir tout"}
                  </button>
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- Activite recente -->
        <div class="activity-section">
          <h3 class="section-title">Activité récente</h3>
          {#if loadingActivity}
            <p class="text-muted">Chargement...</p>
          {:else if activityError}
            <div class="state-banner state-banner-error">
              <span>{activityError}</span>
              <button
                class="btn btn-secondary btn-sm"
                on:click={loadRecentActivity}
              >
                Réessayer
              </button>
            </div>
          {:else if recentActivity.length === 0}
            <p class="text-muted">Aucune activité récente.</p>
          {:else}
            <div class="activity-table-wrap">
              <table class="activity-table">
                <thead>
                  <tr>
                    <th>Action</th>
                    <th>Structure</th>
                    <th>Utilisateur</th>
                    <th>Date</th>
                  </tr>
                </thead>
                <tbody>
                  {#each recentActivity as entry}
                    <tr>
                      <td>
                        <span
                          class="activity-badge"
                          class:activity-override={entry.action ===
                            "overridearchive"}
                        >
                          {entry.action === "overridearchive"
                            ? "Forcé"
                            : "Archivé"}
                        </span>
                      </td>
                      <td class="cell-bold"
                        >{structureName(entry.structure_path)}</td
                      >
                      <td>{entry.user}</td>
                      <td class="cell-date"
                        >{formatTimestamp(entry.timestamp)}</td
                      >
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>
      {:else}
        <div class="empty-state">
          <p>Aucune statistique disponible.</p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- ========================== -->
  <!--   TAB : INVENTAIRE         -->
  <!-- ========================== -->
  {#if activeTab === "inventory"}
    <div class="tab-content">
      <div class="inventory-header">
        <span class="inventory-count"
          >{displayedInventory.length} structure{displayedInventory.length !== 1 ? "s" : ""} affichée{displayedInventory.length !== 1 ? "s" : ""}
          {#if inventoryQuery.trim() || displayedInventory.length !== inventory.length}
            sur {inventory.length}
          {/if}
          dans l'inventaire</span
        >
        <button
          class="btn btn-primary"
          on:click={exportInventoryCSV}
          disabled={displayedInventory.length === 0}
        >
          <svg viewBox="0 0 24 24" fill="none" width="16" height="16"
            ><path
              d="M21 15V19C21 20.1046 20.1046 21 19 21H5C3.89543 21 3 20.1046 3 19V15M7 10L12 15M12 15L17 10M12 15V3"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            /></svg
          >
          Exporter CSV
        </button>
      </div>

      {#if inventoryExportStatus}
        <div
          class="state-banner inventory-export-banner"
          class:state-banner-success={inventoryExportStatusType === "success"}
          class:state-banner-error={inventoryExportStatusType === "error"}
        >
          <span>{inventoryExportStatus}</span>
        </div>
      {/if}

      {#if inventoryError}
        <div class="state-banner state-banner-error">
          <span>{inventoryError}</span>
          <button class="btn btn-secondary btn-sm" on:click={loadInventory}>
            Réessayer
          </button>
        </div>
      {:else if loadingInventory}
        <div class="loading-state"><span class="loader"></span></div>
      {:else if inventory.length > 0}
        <div class="inventory-toolbar">
          <div class="search-field inventory-search-field">
            <svg
              class="search-icon"
              viewBox="0 0 24 24"
              fill="none"
              width="16"
              height="16"
              ><path
                d="M21 21L16.65 16.65M19 11C19 15.4183 15.4183 19 11 19C6.58172 19 3 15.4183 3 11C3 6.58172 6.58172 3 11 3C15.4183 3 19 6.58172 19 11Z"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
              /></svg
            >
            <input
              class="search-input"
              placeholder="Filtrer l'inventaire (structure, opération, type, auteur...)"
              bind:value={inventoryQuery}
              on:input={() => {
                inventoryPage = 1;
                syncUrlState();
              }}
            />
          </div>
          <button
            class="btn btn-secondary btn-sm inventory-reset-btn"
            on:click={resetInventoryFilters}
            disabled={!inventoryHasActiveFilters}
            title="Réinitialiser les filtres"
          >
            Réinitialiser les filtres
          </button>
        </div>
        <div class="inventory-table-shell">
          <button
            type="button"
            class="inventory-scroll-btn inventory-scroll-btn-left"
            on:click={() => scrollInventoryHorizontally("left")}
            disabled={!canScrollInventoryLeft}
            aria-label="Faire défiler le tableau vers la gauche"
            title="Défiler à gauche"
          >
            <svg viewBox="0 0 24 24" fill="none" width="16" height="16" aria-hidden="true"
              ><path
                d="M15 18L9 12L15 6"
                stroke="currentColor"
                stroke-width="2.1"
                stroke-linecap="round"
                stroke-linejoin="round"
              /></svg
            >
          </button>

          <div class="inventory-table-frame">
            <div
              class="table-container inventory-table-container"
              bind:this={inventoryTableContainerEl}
              on:scroll={updateInventoryScrollState}
            >
              <table>
            <thead>
              <tr class="head-row">
                <th class="col-structure">
                  <div class="th-head">
                    <span class="th-title-static">Structure</span>
                  </div>
                </th>
                <th class="col-operation">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("operation_code")}>
                      Opération{inventorySortIcon("operation_code")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("operation_code", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-site">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("operation_site")}>
                      Site{inventorySortIcon("operation_site")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("operation_site", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-type">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("operation_type")}>
                      Opération{inventorySortIcon("operation_type")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("operation_type", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-author">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("operation_responsable")}>
                      Responsable{inventorySortIcon("operation_responsable")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("operation_responsable", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-type">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("structure_type")}>
                      Structure{inventorySortIcon("structure_type")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("structure_type", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-number">
                  <div class="th-head">
                    <button class="th-sort-btn th-sort-btn-with-arrow" type="button" on:click={() => toggleInventorySort("photos_count")}>
                      <span>Photos</span>
                      <span class="th-sort-arrow" class:th-sort-arrow-active={isInventorySortActive("photos_count")} class:th-sort-arrow-desc={isInventorySortDescending("photos_count")} aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" width="12" height="12">
                          <path d="M12 5V19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                          <path d="M7 10L12 5L17 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </span>
                    </button>
                  </div>
                </th>
                <th class="col-number">
                  <div class="th-head">
                    <button class="th-sort-btn th-sort-btn-with-arrow" type="button" on:click={() => toggleInventorySort("photos_total_size_mb")}>
                      <span>Photos Mo</span>
                      <span class="th-sort-arrow" class:th-sort-arrow-active={isInventorySortActive("photos_total_size_mb")} class:th-sort-arrow-desc={isInventorySortDescending("photos_total_size_mb")} aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" width="12" height="12">
                          <path d="M12 5V19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                          <path d="M7 10L12 5L17 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </span>
                    </button>
                  </div>
                </th>
                <th class="col-number">
                  <div class="th-head">
                    <button class="th-sort-btn th-sort-btn-with-arrow" type="button" on:click={() => toggleInventorySort("model_size_mb")}>
                      <span>Modèle Mo</span>
                      <span class="th-sort-arrow" class:th-sort-arrow-active={isInventorySortActive("model_size_mb")} class:th-sort-arrow-desc={isInventorySortDescending("model_size_mb")} aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" width="12" height="12">
                          <path d="M12 5V19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                          <path d="M7 10L12 5L17 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </span>
                    </button>
                  </div>
                </th>
                <th class="col-number">
                  <div class="th-head">
                    <button class="th-sort-btn th-sort-btn-with-arrow" type="button" on:click={() => toggleInventorySort("faces_count")}>
                      <span>Polygones</span>
                      <span class="th-sort-arrow" class:th-sort-arrow-active={isInventorySortActive("faces_count")} class:th-sort-arrow-desc={isInventorySortDescending("faces_count")} aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" width="12" height="12">
                          <path d="M12 5V19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                          <path d="M7 10L12 5L17 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </span>
                    </button>
                  </div>
                </th>
                <th class="col-author">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("model_author")}>
                      Auteur{inventorySortIcon("model_author")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("model_author", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-author">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("depositor")}>
                      Déposant{inventorySortIcon("depositor")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("depositor", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-author">
                  <div class="th-head">
                    <button class="th-sort-btn" type="button" on:click={() => toggleInventorySort("software")}>
                      Logiciel{inventorySortIcon("software")}
                    </button>
                    <button class="th-filter-btn" type="button" on:click|stopPropagation={(e) => toggleInventoryFilterMenu("software", e)}><svg viewBox="0 0 24 24" fill="none" width="12" height="12" aria-hidden="true"><path d="M3 5H21L14 13V19L10 21V13L3 5Z" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg></button>
                  </div>
                </th>
                <th class="col-date">
                  <div class="th-head">
                    <button class="th-sort-btn th-sort-btn-with-arrow" type="button" on:click={() => toggleInventorySort("deposit_date")}>
                      <span>Date dépôt</span>
                      <span class="th-sort-arrow" class:th-sort-arrow-active={isInventorySortActive("deposit_date")} class:th-sort-arrow-desc={isInventorySortDescending("deposit_date")} aria-hidden="true">
                        <svg viewBox="0 0 24 24" fill="none" width="12" height="12">
                          <path d="M12 5V19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                          <path d="M7 10L12 5L17 10" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                        </svg>
                      </span>
                    </button>
                  </div>
                </th>
              </tr>
            </thead>
            <tbody>
              {#each paginatedInventory as item, i}
                <tr class:row-alt={i % 2 === 1}>
                  <td class="cell-bold">{item.structure_id}</td>
                  <td class="cell-code">{item.operation_code}</td>
                  <td>{item.operation_site || "—"}</td>
                  <td>{item.operation_type || "—"}</td>
                  <td>{item.operation_responsable || "—"}</td>
                  <td>{item.structure_type}</td>
                  <td>{item.photos_count || "—"}</td>
                  <td>{item.photos_total_size_mb || "—"}</td>
                  <td>{item.model_size_mb || "—"}</td>
                  <td
                    >{item.faces_count
                      ? parseInt(item.faces_count).toLocaleString("fr-FR")
                      : "—"}</td
                  >
                  <td>{item.model_author || "—"}</td>
                  <td>{item.depositor || "—"}</td>
                  <td>{item.software || "—"}</td>
                  <td class="cell-date">{item.deposit_date || "—"}</td>
                </tr>
              {/each}
            </tbody>
              </table>
            </div>
          </div>

          {#if activeInventoryFilterMenu}
            <div
              class="th-filter-popover th-filter-popover-fixed"
              style="top: {inventoryFilterMenuPos.top}px; left: {inventoryFilterMenuPos.left}px;"
              on:click|stopPropagation
            >
              <div class="th-filter-options">
                {#if activeInventoryFilterMenu === "operation_code"}
                  {#each inventoryOperationCodeOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterOperationCode === option} on:click={() => setInventoryFilterValue("operation_code", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "operation_site"}
                  {#each inventoryOperationSiteOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterOperationSite === option} on:click={() => setInventoryFilterValue("operation_site", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "operation_type"}
                  {#each inventoryOperationTypeOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterOperationType === option} on:click={() => setInventoryFilterValue("operation_type", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "operation_responsable"}
                  {#each inventoryResponsableOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterResponsable === option} on:click={() => setInventoryFilterValue("operation_responsable", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "structure_type"}
                  {#each inventoryStructureTypeOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterStructureType === option} on:click={() => setInventoryFilterValue("structure_type", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "model_author"}
                  {#each inventoryAuthorOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterModelAuthor === option} on:click={() => setInventoryFilterValue("model_author", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "depositor"}
                  {#each inventoryDepositorOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterDepositor === option} on:click={() => setInventoryFilterValue("depositor", option)}>{option}</button>
                  {/each}
                {:else if activeInventoryFilterMenu === "software"}
                  {#each inventorySoftwareOptions as option}
                    <button type="button" class="th-filter-option-btn" class:selected={inventoryFilterSoftware === option} on:click={() => setInventoryFilterValue("software", option)}>{option}</button>
                  {/each}
                {/if}
              </div>
            </div>
          {/if}

          <button
            type="button"
            class="inventory-scroll-btn inventory-scroll-btn-right"
            on:click={() => scrollInventoryHorizontally("right")}
            disabled={!canScrollInventoryRight}
            aria-label="Faire défiler le tableau vers la droite"
            title="Défiler à droite"
          >
            <svg viewBox="0 0 24 24" fill="none" width="16" height="16" aria-hidden="true"
              ><path
                d="M9 18L15 12L9 6"
                stroke="currentColor"
                stroke-width="2.1"
                stroke-linecap="round"
                stroke-linejoin="round"
              /></svg
            >
          </button>
        </div>
        <div class="pagination inventory-pagination">
          <span class="pagination-info">{inventoryPageStart}–{inventoryPageEnd} sur {formatNumber(displayedInventory.length)}</span>
          <div class="page-size-control">
            <label for="archive-inventory-page-size">Par page</label>
            <select
              id="archive-inventory-page-size"
              bind:value={inventoryPerPage}
              on:change={() => {
                inventoryPage = 1;
                syncUrlState();
              }}
            >
              <option value={25}>25</option>
              <option value={50}>50</option>
              <option value={100}>100</option>
            </select>
          </div>
          <div class="pagination-controls">
            <button
              class="btn btn-secondary btn-sm"
              aria-label="Page précédente"
              title="Page précédente"
              disabled={inventoryPage <= 1}
              on:click={() => {
                inventoryPage--;
                syncUrlState();
              }}
            >
              <svg viewBox="0 0 24 24" fill="none" width="14" height="14"
                ><path
                  d="M15 18L9 12L15 6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /></svg
              >
            </button>
            <span class="page-num">Page {inventoryPage} / {inventoryTotalPages}</span>
            <button
              class="btn btn-secondary btn-sm"
              aria-label="Page suivante"
              title="Page suivante"
              disabled={inventoryPage >= inventoryTotalPages}
              on:click={() => {
                inventoryPage++;
                syncUrlState();
              }}
            >
              <svg viewBox="0 0 24 24" fill="none" width="14" height="14"
                ><path
                  d="M9 18L15 12L9 6"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                /></svg
              >
            </button>
          </div>
        </div>
        {#if displayedInventory.length === 0}
          <div class="empty-state">
            <p>Aucun résultat ne correspond à la recherche dans l'inventaire.</p>
          </div>
        {/if}
      {:else}
        <div class="empty-state">
          <p>Aucune donnée d'inventaire.</p>
        </div>
      {/if}
    </div>
  {/if}
</main>

<style>
  * {
    box-sizing: border-box;
  }

  .archive-page {
    padding: var(--spacing-2xl);
    max-width: 1400px;
    margin: 0 auto;
  }

  .archive-page.archive-page-inventory {
    max-width: min(96vw, 1820px);
  }

  .page-header {
    margin-bottom: var(--spacing-lg);
  }

  .page-header h1 {
    margin: 0;
    font-size: var(--font-size-xl);
    font-weight: 700;
    color: var(--color-neutral-900);
  }

  .header-subtitle {
    margin: var(--spacing-xs) 0 0;
    color: var(--color-neutral-600);
    font-size: var(--font-size-sm);
  }

  .tabs-bar {
    margin-bottom: var(--spacing-xl);
    border-bottom: 1px solid var(--color-neutral-400);
  }

  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0;
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-xl);
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    background: transparent;
    cursor: pointer;
    font-size: var(--font-size-sm);
    font-weight: 500;
    font-family: var(--font-family);
    color: var(--color-neutral-700);
    transition: all 0.2s;
  }

  .tab:hover {
    color: var(--color-neutral-900);
  }

  .tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
    font-weight: 600;
  }

  .tab svg {
    opacity: 0.6;
  }

  .tab.active svg {
    opacity: 1;
  }

  .tab-content {
    animation: fadeIn 0.2s ease;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .state-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-error-border);
    background: var(--color-error-bg);
    color: var(--color-error);
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  .state-banner-error :global(.btn.btn-secondary) {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-400);
    color: var(--color-neutral-800);
  }

  .state-banner-success {
    border-color: var(--color-success-border);
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .inventory-export-banner {
    margin-bottom: var(--spacing-md);
  }

  .toolbar {
    margin-bottom: var(--spacing-xl);
  }

  .search-row {
    display: flex;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-md);
    align-items: center;
  }

  .search-field {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
    min-width: 0;
    width: 100%;
  }

  .search-icon {
    position: absolute;
    left: var(--spacing-md);
    color: var(--color-neutral-600);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    border-radius: var(--border-radius-md);
    border: 1px solid var(--color-neutral-400);
    padding: var(--spacing-md) var(--spacing-lg);
    padding-left: 2.5rem;
    font-size: var(--font-size-sm);
    font-family: var(--font-family);
    background: var(--color-neutral-100);
    color: var(--color-neutral-900);
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(30, 58, 95, 0.1);
  }

  .consultation-toolbar {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  .search-submit-btn {
    min-height: 42px;
    padding: 0 1rem;
    white-space: nowrap;
  }

  .consultation-extra-filters {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
    width: 100%;
  }

  .consultation-filter-chip {
    display: block;
    min-width: 0;
  }

  .consultation-filter-chip :global(.meta-select-wrapper) {
    min-width: 0;
    width: 100%;
  }

  .consultation-filter-chip :global(.chip-select) {
    width: 100%;
    min-width: 0;
    max-width: none;
    min-height: 28px;
    border-radius: 999px;
    border: 1px solid var(--color-neutral-300);
    background: #ffffff;
    font-size: 0.74rem;
    line-height: 1.1;
    padding: 3px 24px 3px 9px;
    color: var(--color-neutral-700);
    box-shadow: none;
  }

  .btn-link {
    border: none;
    background: none;
    color: var(--color-neutral-600);
    font-size: var(--font-size-xs);
    font-family: var(--font-family);
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
  }

  .btn-link:hover {
    color: var(--color-primary);
  }

  .consultation-layout {
    display: grid;
    grid-template-columns: minmax(260px, 340px) minmax(0, 1fr);
    gap: var(--spacing-md);
    min-height: 520px;
  }

  .consultation-list-panel,
  .consultation-detail-panel {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-400);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--shadow-sm);
  }

  .consultation-list-panel {
    padding: var(--spacing-sm);
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .consultation-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: auto;
    max-height: min(64vh, 760px);
    padding-right: 2px;
  }

  .consultation-item {
    width: 100%;
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    background: var(--color-neutral-100);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 8px 10px;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }

  .consultation-item:hover {
    border-color: var(--color-primary);
    background: var(--color-neutral-200);
  }

  .consultation-item.selected {
    border-color: var(--color-primary);
    background: rgba(30, 58, 95, 0.08);
  }

  .consultation-item-main {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .consultation-item-structure {
    font-size: var(--font-size-sm);
    font-weight: 700;
    color: var(--color-primary);
    line-height: 1.25;
  }

  .consultation-item-meta {
    font-size: var(--font-size-xs);
    color: var(--color-neutral-700);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .consultation-detail-panel {
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    min-width: 0;
  }

  .consultation-detail-empty {
    min-height: 280px;
    display: grid;
    place-items: center;
    color: var(--color-neutral-600);
    text-align: center;
    border: 1px dashed var(--color-neutral-400);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-xl);
  }

  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-sm);
    border-bottom: 1px solid var(--color-neutral-300);
    padding-bottom: var(--spacing-sm);
  }

  .detail-header-right {
    display: inline-flex;
    flex-direction: row;
    align-items: flex-start;
    gap: var(--spacing-sm);
    justify-content: flex-start;
    margin-left: auto;
  }

  .detail-assets-summary {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    width: fit-content;
    margin-top: 8px;
    padding: 4px 8px;
    border-radius: 999px;
    border: 1px solid var(--color-neutral-300);
    background: rgba(248, 250, 252, 0.9);
  }

  .detail-header-actions {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .detail-title-wrap h2 {
    margin: 0;
    font-size: var(--font-size-lg);
    color: var(--color-primary);
    line-height: 1.2;
  }

  .detail-title-wrap p {
    margin: var(--spacing-xs) 0 0;
    color: var(--color-neutral-700);
    font-size: var(--font-size-sm);
  }

  .detail-open-path-btn {
    min-height: 36px;
    padding: 0 12px;
    border-radius: 10px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    white-space: nowrap;
  }

  .detail-mobile-close-btn {
    display: none;
    min-height: 36px;
    width: 36px;
    padding: 0;
    border-radius: 10px;
    align-items: center;
    justify-content: center;
    line-height: 1;
    font-size: 1rem;
  }

  .consultation-mobile-overlay {
    display: none;
  }

  .detail-action-status {
    margin: 0;
    color: var(--color-neutral-700);
    font-size: var(--font-size-xs);
  }

  .detail-section {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    padding: var(--spacing-md);
  }

  .detail-section h3 {
    margin: 0 0 var(--spacing-sm);
    color: var(--color-neutral-900);
    font-size: var(--font-size-sm);
    font-weight: 700;
  }

  .detail-viewers-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: var(--spacing-md);
    align-items: stretch;
  }

  .detail-viewer-card {
    min-height: 240px;
  }

  .detail-viewers-grid-empty .detail-viewer-card {
    min-height: 120px;
  }

  .detail-viewer-card-compact {
    padding-bottom: var(--spacing-sm);
  }

  .detail-muted {
    margin: 0;
    color: var(--color-neutral-600);
    font-size: var(--font-size-sm);
  }

  .detail-loading {
    padding: var(--spacing-xl);
  }

  .ortho-preview {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .ortho-preview-thumb-wrap {
    position: relative;
  }

  .ortho-viewer-icon-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    border: 1px solid rgba(148, 163, 184, 0.55);
    background: rgba(255, 255, 255, 0.88);
    color: var(--color-neutral-700);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 2;
  }

  .ortho-preview-thumb {
    display: block;
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-sm);
    overflow: hidden;
    background: var(--color-neutral-200);
    width: 100%;
    cursor: zoom-in;
    padding: 0;
  }

  .ortho-preview-thumb img {
    display: block;
    width: 100%;
    height: 180px;
    object-fit: contain;
    background: #ffffff;
  }

  .ortho-preview-caption {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    color: var(--color-neutral-700);
    font-size: var(--font-size-xs);
  }

  .metadata-panel {
    border-color: rgba(203, 213, 225, 0.8);
    border-radius: 14px;
    background: #ffffff;
    box-shadow: 0 8px 20px rgba(15, 23, 42, 0.05);
    padding: 10px 12px;
  }

  .metadata-panel-header {
    margin-bottom: 8px;
  }

  .metadata-panel-header > h3 {
    margin: 0;
    font-size: 1rem;
    letter-spacing: 0;
  }

  .metadata-panel-header > p {
    margin: 3px 0 0;
    color: var(--color-neutral-600);
    font-size: 0.73rem;
    line-height: 1.35;
  }

  .ortho-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.82);
    display: grid;
    place-items: center;
    z-index: 1400;
    padding: var(--spacing-lg);
  }

  .ortho-modal-backdrop img {
    max-width: min(94vw, 1200px);
    max-height: 88vh;
    border-radius: var(--border-radius-md);
    border: 1px solid rgba(255, 255, 255, 0.25);
    background: #ffffff;
    object-fit: contain;
  }

  .ortho-modal-close {
    position: absolute;
    top: var(--spacing-lg);
    right: var(--spacing-lg);
    border: 1px solid rgba(255, 255, 255, 0.45);
    background: rgba(15, 23, 42, 0.7);
    color: #fff;
    border-radius: var(--border-radius-sm);
    width: 36px;
    height: 36px;
    font-size: 1.4rem;
    line-height: 1;
    cursor: pointer;
  }

  .consultation-pagination {
    margin-top: var(--spacing-sm);
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    flex-wrap: nowrap;
  }

  .consultation-pagination .page-size-control {
    align-self: center;
  }

  .consultation-pagination .pagination-controls {
    align-self: center;
    margin-left: 0;
  }

  .inventory-table-shell {
    position: relative;
  }

  .inventory-scroll-btn {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    z-index: 5;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: var(--color-neutral-900);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: opacity 0.15s ease, color 0.15s ease, transform 0.12s ease;
  }

  .inventory-scroll-btn-left {
    left: -34px;
  }

  .inventory-scroll-btn-right {
    right: -34px;
  }

  .inventory-scroll-btn:hover:not(:disabled) {
    color: var(--color-primary);
    transform: translateY(-50%) scale(1.06);
  }

  .inventory-scroll-btn:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .inventory-scroll-btn svg {
    width: 23px;
    height: 23px;
  }

  .inventory-table-container {
    scroll-behavior: smooth;
  }

  .inventory-table-frame {
    position: relative;
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-400);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--shadow-sm);
    overflow: visible;
  }

  .table-container {
    background: transparent;
    border: none;
    border-radius: var(--border-radius-lg);
    box-shadow: none;
    overflow-x: auto;
    overflow-y: visible;
  }

  .table-container > table {
    width: 100%;
    min-width: 1380px;
    border-collapse: collapse;
    font-size: var(--font-size-sm);
  }

  .table-container th {
    text-align: left;
    padding: 9px 10px;
    background: var(--color-neutral-300);
    position: sticky;
    top: 0;
    z-index: 1;
    font-weight: 700;
    color: var(--color-neutral-900);
    font-size: var(--font-size-xs);
    text-transform: none;
    letter-spacing: 0.01em;
    white-space: nowrap;
    position: sticky;
    position: -webkit-sticky;
  }

  .table-container .head-row th {
    top: 0;
    z-index: 2;
    border-right: 1px solid rgba(148, 163, 184, 0.35);
  }

  .table-container .head-row th:last-child {
    border-right: none;
  }

  .th-head {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: 100%;
  }

  .th-sort-btn {
    border: none;
    background: transparent;
    padding: 0;
    margin: 0;
    width: 100%;
    text-align: left;
    font-size: inherit;
    font-family: var(--font-family);
    font-weight: inherit;
    color: inherit;
    cursor: pointer;
  }

  .th-sort-btn:hover {
    color: var(--color-primary);
  }

  .th-sort-btn-with-arrow {
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .th-sort-btn-with-arrow > span:first-child {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .th-sort-arrow {
    color: #183353;
    flex-shrink: 0;
    min-width: 0.9rem;
    height: 0.9rem;
    text-align: center;
    opacity: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transform: rotate(180deg);
    transition: transform 0.15s ease, opacity 0.15s ease, color 0.15s ease;
  }

  .th-sort-arrow svg path {
    stroke-width: 2.8;
  }

  .th-sort-arrow.th-sort-arrow-active {
    color: #102743;
    opacity: 1;
  }

  .th-sort-arrow.th-sort-arrow-active:not(.th-sort-arrow-desc) {
    transform: rotate(0deg);
  }

  .th-sort-arrow.th-sort-arrow-active.th-sort-arrow-desc {
    transform: rotate(180deg);
  }

  .th-sort-btn-with-arrow:hover .th-sort-arrow {
    color: var(--color-primary);
  }

  .th-sort-btn-with-arrow:hover .th-sort-arrow.th-sort-arrow-active {
    opacity: 1;
  }

  .th-title-static {
    font-size: inherit;
    font-weight: inherit;
    color: inherit;
  }

  .th-filter-btn {
    border: none;
    background: transparent;
    color: var(--color-neutral-900);
    width: auto;
    height: auto;
    line-height: 1;
    padding: 0 0 0 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .th-filter-btn:hover {
    color: var(--color-primary);
  }

  .th-filter-btn svg {
    width: 14px;
    height: 14px;
  }

  .th-filter-btn svg path {
    stroke-width: 2.35 !important;
  }

  .head-row th {
    position: sticky;
  }

  .head-row th {
    overflow: visible;
  }

  .th-filter-popover {
    position: absolute;
    top: calc(100% + 6px);
    right: 8px;
    z-index: 8;
    min-width: 130px;
    background: #ffffff;
    border: 1px solid var(--color-neutral-300);
    border-radius: 8px;
    box-shadow: 0 8px 18px rgba(15, 23, 42, 0.12);
    padding: 4px;
  }

  .th-filter-popover-fixed {
    position: fixed;
    top: unset;
    right: unset;
    z-index: 9999;
    transform: translateX(-100%);
  }

  .th-filter-options {
    display: flex;
    flex-direction: column;
    max-height: 220px;
    overflow: auto;
  }

  .th-filter-option-btn {
    border: none;
    background: transparent;
    color: var(--color-neutral-800);
    text-align: left;
    font: inherit;
    font-size: 0.78rem;
    padding: 6px 8px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
  }

  .th-filter-option-btn:hover {
    background: var(--color-neutral-200);
  }

  .th-filter-option-btn.selected {
    background: rgba(30, 58, 95, 0.1);
    color: var(--color-primary);
    font-weight: 700;
  }

  .table-container td {
    padding: 7px 10px;
    border-top: 1px solid var(--color-neutral-300);
    color: var(--color-neutral-800);
    font-size: 0.84rem;
  }

  .table-container tr.row-alt {
    background: var(--color-neutral-200);
  }

  .col-operation {
    width: 96px;
  }

  .col-structure {
    width: 104px;
  }

  .col-type {
    width: 116px;
  }

  .col-site {
    min-width: 132px;
  }

  .col-author {
    min-width: 138px;
  }

  .col-date {
    width: 106px;
  }

  .col-number {
    width: 94px;
  }

  .cell-code {
    font-weight: 700;
    color: var(--color-primary);
  }

  .cell-bold {
    font-weight: 700;
  }

  .cell-date {
    color: var(--color-neutral-600);
    font-size: var(--font-size-xs);
    white-space: nowrap;
  }

  .cell-badges {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .asset-badge {
    display: inline-block;
    padding: 1px 7px;
    border-radius: var(--border-radius-sm);
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .asset-3d {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .asset-ortho {
    background: var(--color-info-bg);
    color: var(--color-info);
  }

  .asset-prod {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }

  .pagination {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: var(--spacing-md);
    padding: 0 var(--spacing-xs);
    flex-wrap: wrap;
  }

  .inventory-pagination {
    margin-top: var(--spacing-lg);
  }

  .pagination-info {
    font-size: var(--font-size-sm);
    color: var(--color-neutral-600);
  }

  .pagination-controls {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-left: auto;
  }

  .page-num {
    min-width: 96px;
    text-align: center;
    font-size: var(--font-size-sm);
    color: var(--color-neutral-700);
  }

  .page-size-control {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-xs);
    color: var(--color-neutral-700);
    font-size: var(--font-size-xs);
    font-weight: 600;
  }

  .page-size-control label {
    white-space: nowrap;
  }

  .page-size-control select {
    min-height: 30px;
    border-radius: var(--border-radius-sm);
    border: 1px solid var(--color-neutral-400);
    background: var(--color-neutral-100);
    color: var(--color-neutral-800);
    font-size: var(--font-size-xs);
    font-family: var(--font-family);
    padding: 2px 8px;
  }

  .btn-sm {
    padding: var(--spacing-xs) var(--spacing-sm) !important;
    min-width: 32px;
  }

  .stat-cards {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-2xl);
  }

  .stat-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-lg);
    padding: var(--spacing-xl);
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-400);
    border-radius: var(--border-radius-lg);
    box-shadow: var(--shadow-sm);
  }

  .stat-icon {
    width: 44px;
    height: 44px;
    border-radius: var(--border-radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .icon-structures {
    background: rgba(30, 58, 95, 0.08);
    color: var(--color-primary);
  }

  .icon-operations {
    background: rgba(193, 122, 92, 0.1);
    color: var(--color-secondary);
  }

  .icon-size {
    background: var(--color-info-bg);
    color: var(--color-info);
  }

  .icon-models {
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .icon-archive-size {
    background: rgba(71, 85, 105, 0.12);
    color: #334155;
  }

  .stat-body {
    display: flex;
    flex-direction: column;
  }

  .stat-value {
    font-size: var(--font-size-xl);
    font-weight: 700;
    color: var(--color-neutral-900);
    line-height: 1.2;
  }

  .stat-label {
    margin-top: 2px;
    color: var(--color-neutral-600);
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .breakdowns-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-2xl);
  }

  .breakdown-card {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-400);
    border-radius: var(--border-radius-lg);
    overflow: hidden;
    box-shadow: var(--shadow-sm);
  }

  .breakdown-title {
    margin: 0;
    padding: var(--spacing-md) var(--spacing-xl);
    border-bottom: 1px solid var(--color-neutral-400);
    background: var(--color-neutral-300);
    color: var(--color-neutral-700);
    font-size: var(--font-size-xs);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .bar-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    max-height: 280px;
    overflow-y: auto;
    padding: var(--spacing-md) var(--spacing-xl);
  }

  .bar-row {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .bar-row-btn {
    width: 100%;
    border: none;
    background: none;
    padding: 0;
    cursor: pointer;
    text-align: left;
  }

  .bar-row-btn:hover .bar-label,
  .bar-row-btn:focus-visible .bar-label {
    color: var(--color-primary);
    text-decoration: underline;
  }

  .bar-row-btn:focus-visible {
    outline: 2px solid rgba(30, 58, 95, 0.35);
    outline-offset: 2px;
    border-radius: 4px;
  }

  .bar-label {
    width: 130px;
    flex-shrink: 0;
    text-align: right;
    color: var(--color-neutral-800);
    font-size: var(--font-size-xs);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bar-track {
    flex: 1;
    height: 8px;
    border-radius: 4px;
    background: var(--color-neutral-300);
    overflow: hidden;
  }

  .bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width 0.5s ease;
    min-width: 3px;
  }

  .bar-primary {
    background: var(--color-primary);
  }

  .bar-secondary {
    background: var(--color-secondary);
  }

  .bar-info {
    background: var(--color-info);
  }

  .bar-success {
    background: var(--color-success);
  }

  .bar-value {
    width: 32px;
    text-align: right;
    color: var(--color-neutral-700);
    font-size: var(--font-size-xs);
    font-weight: 700;
  }

  .breakdown-actions {
    border-top: 1px solid var(--color-neutral-300);
    padding: var(--spacing-xs) var(--spacing-xl) var(--spacing-sm);
    display: flex;
    justify-content: flex-end;
  }

  .section-title {
    margin: 0 0 var(--spacing-md);
    color: var(--color-neutral-900);
    font-size: var(--font-size-sm);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .activity-section {
    background: var(--color-neutral-100);
    border: 1px solid var(--color-neutral-400);
    border-radius: var(--border-radius-lg);
    padding: var(--spacing-xl);
    box-shadow: var(--shadow-sm);
  }

  .activity-table-wrap {
    border: 1px solid var(--color-neutral-300);
    border-radius: var(--border-radius-md);
    overflow: auto;
  }

  .activity-table {
    width: 100%;
    min-width: 620px;
    border-collapse: collapse;
    font-size: var(--font-size-sm);
  }

  .activity-table th {
    padding: var(--spacing-sm) var(--spacing-lg);
    background: var(--color-neutral-200);
    font-size: var(--font-size-xs);
  }

  .activity-table td {
    padding: var(--spacing-sm) var(--spacing-lg);
    border-top: 1px solid var(--color-neutral-300);
  }

  .activity-badge {
    display: inline-block;
    padding: 1px 8px;
    border-radius: var(--border-radius-sm);
    font-size: var(--font-size-xs);
    font-weight: 600;
    background: var(--color-success-bg);
    color: var(--color-success);
  }

  .activity-badge.activity-override {
    background: var(--color-warning-bg);
    color: var(--color-warning);
  }

  .inventory-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-xl);
  }

  .inventory-count {
    color: var(--color-neutral-700);
    font-size: var(--font-size-sm);
    font-weight: 500;
  }

  .inventory-toolbar {
    margin-bottom: var(--spacing-md);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
  }

  .inventory-reset-btn {
    min-height: 30px;
    padding: 0 10px;
    font-size: 0.74rem;
    white-space: nowrap;
  }

  .inventory-search-field {
    max-width: 620px;
    width: 100%;
  }

  .loading-state {
    display: flex;
    justify-content: center;
    padding: calc(var(--spacing-2xl) * 2);
  }

  .loader {
    display: inline-block;
    width: 28px;
    height: 28px;
    border: 3px solid var(--color-neutral-400);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-2xl);
    color: var(--color-neutral-600);
    background: var(--color-neutral-100);
    border-radius: var(--border-radius-lg);
    border: 1px solid var(--color-neutral-400);
  }

  .empty-state svg {
    margin-bottom: var(--spacing-md);
    color: var(--color-neutral-500);
    opacity: 0.4;
  }

  .empty-state p {
    margin: 0;
    font-size: var(--font-size-sm);
  }

  .text-muted {
    color: var(--color-neutral-600);
    font-size: var(--font-size-sm);
    font-style: italic;
  }

  @media (max-width: 1200px) {
    .stat-cards {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .breakdowns-grid {
      grid-template-columns: 1fr;
    }

    .consultation-layout {
      grid-template-columns: minmax(250px, 320px) minmax(0, 1fr);
    }
  }

  @media (max-width: 980px) {
    .archive-page {
      padding: var(--spacing-xl);
    }

    .consultation-layout {
      grid-template-columns: 1fr;
      min-height: 0;
    }

    .consultation-list {
      max-height: min(44vh, 460px);
    }

    .consultation-layout.consultation-layout-empty .consultation-detail-panel {
      display: none;
    }

    .consultation-layout.consultation-layout-has-selection .consultation-detail-panel {
      position: fixed;
      top: 12px;
      right: 12px;
      bottom: 12px;
      left: 12px;
      z-index: 1301;
      margin: 0;
      border-radius: 14px;
      box-shadow: 0 20px 48px rgba(15, 23, 42, 0.26);
      overflow: auto;
      overscroll-behavior: contain;
      background: var(--color-neutral-100);
    }

    .consultation-mobile-overlay {
      display: block;
      position: fixed;
      inset: 0;
      z-index: 1300;
      border: none;
      padding: 0;
      background: rgba(15, 23, 42, 0.42);
      cursor: pointer;
    }

    .detail-mobile-close-btn {
      display: inline-flex;
    }

    .consultation-detail-empty {
      min-height: 200px;
    }

    .detail-viewers-grid {
      grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    }
  }

  @media (max-width: 760px) {
    .search-row {
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      align-items: stretch;
      gap: 8px;
    }

    .search-submit-btn {
      min-height: 40px;
      align-self: stretch;
    }

    .consultation-extra-filters {
      gap: 6px;
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }

    .consultation-filter-chip {
      width: auto;
    }

    .consultation-filter-chip :global(.chip-select) {
      width: 100%;
      max-width: none;
      min-height: 26px;
      font-size: 0.72rem;
      padding: 3px 22px 3px 8px;
    }

    .pagination {
      flex-direction: column;
      align-items: flex-start;
    }

    .consultation-pagination .pagination-controls {
      justify-content: center;
    }

    .consultation-item {
      align-items: flex-start;
      flex-direction: column;
    }

    .detail-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .detail-header-right {
      width: 100%;
      align-items: flex-start;
      margin-left: 0;
    }

    .metadata-panel-header > p {
      font-size: 0.74rem;
    }

    .inventory-header {
      flex-direction: column;
      align-items: flex-start;
    }

    .inventory-toolbar {
      flex-direction: column;
      align-items: stretch;
    }

    .inventory-scroll-btn {
      width: 30px;
      height: 30px;
    }

    .consultation-layout.consultation-layout-has-selection .detail-header {
      flex-direction: row;
      align-items: flex-start;
      justify-content: space-between;
      gap: var(--spacing-sm);
    }

    .consultation-layout.consultation-layout-has-selection .detail-header-right {
      width: auto;
      align-items: flex-start;
      margin-left: auto;
    }

    .consultation-layout.consultation-layout-has-selection .detail-open-path-btn {
      width: auto;
      justify-content: center;
    }
  }

  @media (max-width: 640px) {
    .archive-page {
      padding: var(--spacing-md);
    }

    .search-row {
      grid-template-columns: 1fr;
    }

    .search-submit-btn {
      width: 100%;
    }

    .consultation-extra-filters {
      grid-template-columns: 1fr;
    }

    .consultation-pagination {
      gap: 10px;
    }

    .tab {
      padding: var(--spacing-sm) var(--spacing-md);
    }

    .activity-section {
      padding: var(--spacing-md);
    }

    .stat-card {
      padding: var(--spacing-md);
    }

    .inventory-scroll-btn {
      width: 28px;
      height: 28px;
    }
  }

  @media (max-width: 520px) {
    .consultation-pagination {
      flex-wrap: wrap;
      justify-content: center;
      gap: 8px;
    }

    .consultation-pagination .page-size-control {
      width: 100%;
      justify-content: center;
    }

    .consultation-pagination .pagination-controls {
      width: 100%;
      justify-content: center;
    }
  }

  @media (max-width: 480px) {
    .archive-page {
      padding: var(--spacing-sm);
    }
  }
</style>
