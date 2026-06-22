import { invoke } from '@tauri-apps/api/core'
import type { Instance } from '~/types/launcher'
import type {
  ModrinthSearchParams,
  ModrinthSearchResponse,
  ModrinthVersion,
  ModrinthCategory,
  ModrinthProjectType,
  ModrinthProjectFull,
  InstalledItem,
  ModUpdate,
  ModpackUpdate,
} from '~/types/modrinth'

/** Thin typed wrappers around the Rust Modrinth commands. */
export const useModrinth = () => {
  const search = (params: ModrinthSearchParams) =>
    invoke<ModrinthSearchResponse>('modrinth_search', { params })

  const versions = (projectId: string, loaders?: string[], gameVersions?: string[]) =>
    invoke<ModrinthVersion[]>('modrinth_versions', {
      projectId,
      loaders: loaders ?? null,
      gameVersions: gameVersions ?? null,
    })

  /** Full project incl. the markdown `body`. */
  const project = (id: string) => invoke<ModrinthProjectFull>('modrinth_project', { id })

  const categories = (projectType: ModrinthProjectType) =>
    invoke<ModrinthCategory[]>('modrinth_categories', { projectType })

  /** Installs a version (+ its required dependencies) into an instance and
   *  records them in the instance's content index. Returns the newly-added items. */
  const installWithDeps = (
    instanceId: string,
    versionId: string,
    gameVersion?: string,
    loader?: string,
  ) =>
    invoke<InstalledItem[]>('modrinth_install_with_deps', {
      instanceId,
      versionId,
      gameVersion: gameVersion ?? null,
      loader: loader ?? null,
    })

  /** Content already installed in an instance (from the content index). */
  const getInstalled = (instanceId: string) =>
    invoke<InstalledItem[]>('get_installed_content', { instanceId })

  /** Links local jars to Modrinth by file hash; returns how many were matched. */
  const matchLocal = (instanceId: string) =>
    invoke<number>('match_local_mods', { instanceId })

  /** Matches one local jar by sha1; returns whether it matched. */
  const matchFile = (instanceId: string, filename: string) =>
    invoke<boolean>('modrinth_match_file', { instanceId, filename })

  /** Installed mods that have a newer compatible version available. */
  const checkUpdates = (instanceId: string, loaders?: string[], gameVersions?: string[]) =>
    invoke<ModUpdate[]>('check_mod_updates', {
      instanceId,
      loaders: loaders ?? null,
      gameVersions: gameVersions ?? null,
    })

  /** Creates a new instance from a `.mrpack` URL (downloads the icon too). */
  const installModpack = (
    url: string,
    nameOverride?: string | null,
    iconUrl?: string | null,
    projectId?: string | null,
    versionId?: string | null,
  ) =>
    invoke<Instance>('modrinth_install_modpack', {
      url,
      nameOverride: nameOverride ?? null,
      iconUrl: iconUrl ?? null,
      projectId: projectId ?? null,
      versionId: versionId ?? null,
    })

  /** Newer version of an instance's modpack, if available. */
  const checkModpackUpdate = (instanceId: string) =>
    invoke<ModpackUpdate | null>('check_modpack_update', { instanceId })

  /** Updates an instance's modpack to the latest version. */
  const updateModpack = (instanceId: string) =>
    invoke<void>('update_modpack', { instanceId })

  return { search, versions, project, categories, installWithDeps, getInstalled, matchLocal, matchFile, checkUpdates, installModpack, checkModpackUpdate, updateModpack }
}
