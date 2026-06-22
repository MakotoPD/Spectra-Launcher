import type { ContentKind, ModrinthProjectType } from '~/types/modrinth'

/** The real Modrinth project_type to search for a given UI content kind. */
export function searchProjectType(kind: ContentKind): ModrinthProjectType {
  return kind === 'datapack' ? 'mod' : kind
}

/** Category facets that define the pseudo-types (datapacks are mods + a category). */
export function baseCategories(kind: ContentKind): string[] {
  return kind === 'datapack' ? ['datapack'] : []
}

/** Whether a mod-loader filter (fabric/forge/…) is meaningful for this kind. */
export function usesLoaderFilter(kind: ContentKind): boolean {
  return kind === 'mod' || kind === 'modpack'
}

/** The loader name Modrinth uses when listing versions for this kind. */
export function loaderFacetFor(kind: ContentKind, loader?: string): string[] {
  if (kind === 'datapack') return ['datapack']
  if (usesLoaderFilter(kind) && loader && loader !== 'vanilla') return [loader]
  return []
}

/** Compact download counts, e.g. 12345 -> "12.3k". */
export function compactNumber(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`
  return String(n)
}
