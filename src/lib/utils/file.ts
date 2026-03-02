export function filename(path: string): string {
    return path.split(/[/\\]/).pop() || path;
}
