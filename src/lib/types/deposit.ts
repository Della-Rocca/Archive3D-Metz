export interface OperationMeta {
    code: string;
    site: string;
    op_type: string;
    responsable: string;
}

export interface StructureMeta {
    id: string;
    st_type: string;
    description: string;
    model_author: string;
    depositor: string;
    photos_count: string;
    faces_count: string;
    software: string;
}

export interface DepositMetadata {
    operation: OperationMeta;
    structure: StructureMeta;
}

export interface StructureDetails {
    metadata: DepositMetadata | null;
    models: string[];
    orthos: string[];
    photos: string[];
    work_files: string[];
}

export interface Presets {
    operations: OperationMeta[];
    structure_types: string[];
    operation_types: string[];
    software_types: string[];
    sites: string[];
    responsables: string[];
    model_authors: string[];
    depositors: string[];
}
