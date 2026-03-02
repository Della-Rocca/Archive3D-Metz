import { invoke } from "@tauri-apps/api/core";
import type { Presets, DepositMetadata } from "$lib/types/deposit";

export async function getMetadataPresets(): Promise<Presets> {
    return invoke<Presets>("get_metadata_presets");
}

export async function countModelPolygons(path: string): Promise<number> {
    return invoke<number>("count_model_polygons", { path });
}

export async function depositStructure(
    metadata: DepositMetadata,
    files: {
        model: string[];
        ortho: string[];
        photo: string[];
        work: string[];
    }
): Promise<string> {
    return invoke<string>("deposit_structure", {
        metadata,
        modelFiles: files.model,
        orthoFiles: files.ortho,
        photoFiles: files.photo,
        workFiles: files.work,
    });
}
