import { invoke } from '@tauri-apps/api/core'
import type { Instance } from '~/types/launcher'
import type {
  ModrinthSearchParams,
  ModrinthSearchResponse,
  ModrinthVersion,
  ModrinthProjectFull,
  ModrinthCategory,
  InstalledItem,
} from '~/types/modrinth'

/** A mod CurseForge refuses to serve to third-party launchers (mirrors BlockedMod). */
export interface BlockedMod {
  name: string
  file_id: string
  url: string
}

export interface CfInstallResult {
  added: InstalledItem[]
  blocked: BlockedMod[]
}

/**
 * Thin typed wrappers around the Rust CurseForge commands. Results are
 * normalized to the same shapes the Modrinth browser consumes.
 */
export const useCurseforge = () => {
  const enabled = () => invoke<boolean>('cf_enabled')

  const search = (params: ModrinthSearchParams) =>
    invoke<ModrinthSearchResponse>('curseforge_search', { params })

  const versions = (projectId: string, loaders?: string[], gameVersions?: string[]) =>
    invoke<ModrinthVersion[]>('curseforge_versions', {
      projectId,
      loaders: loaders ?? null,
      gameVersions: gameVersions ?? null,
    })

  const project = (id: string) => invoke<ModrinthProjectFull>('curseforge_project', { id })

  /** Categories for the kind's classId ({ name, header=numeric id }). */
  const categories = (projectType: string) =>
    invoke<ModrinthCategory[]>('curseforge_categories', { projectType })

  /** Installs a mod (+ required deps); returns added items and any blocked mods. */
  const installWithDeps = (
    instanceId: string,
    projectId: string,
    fileId: string,
    gameVersion?: string,
    loader?: string,
  ) =>
    invoke<CfInstallResult>('curseforge_install_with_deps', {
      instanceId,
      projectId,
      fileId,
      gameVersion: gameVersion ?? null,
      loader: loader ?? null,
    })

  /** Links local jars to CurseForge by fingerprint; returns how many matched. */
  const matchLocal = (instanceId: string) =>
    invoke<number>('curseforge_match_local', { instanceId })

  /** Matches one local jar by fingerprint; returns whether it matched. */
  const matchFile = (instanceId: string, filename: string) =>
    invoke<boolean>('curseforge_match_file', { instanceId, filename })

  /** Creates a new instance from a CurseForge modpack (by project + file id). */
  const installModpack = (projectId: string, fileId: string, nameOverride?: string | null) =>
    invoke<Instance>('curseforge_install_modpack', {
      projectId,
      fileId,
      nameOverride: nameOverride ?? null,
    })

  return { enabled, search, versions, project, categories, installWithDeps, matchLocal, matchFile, installModpack }
}
