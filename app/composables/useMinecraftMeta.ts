import { invoke } from '@tauri-apps/api/core'
import type { LoaderType } from '~/types/launcher'

export interface MinecraftVersion {
  id: string
  kind: string // "release" | "snapshot" | ...
  release_time: string
}

export interface LoaderVersion {
  version: string
  stable: boolean
}

export type LoaderVersionMode = 'stable' | 'latest' | 'other'

/**
 * Version lists for the create-instance pickers. The actual game files are
 * downloaded by Lyceris on launch — these are just the selectable versions.
 */
export const useMinecraftMeta = () => {
  const getMinecraftVersions = (includeSnapshots = false) =>
    invoke<MinecraftVersion[]>('get_minecraft_versions', { includeSnapshots })

  const getLoaderVersions = (loader: LoaderType, mcVersion: string) =>
    invoke<LoaderVersion[]>('get_loader_versions', { loader, mcVersion })

  /**
   * Resolves the loader version string to hand to the backend.
   * - stable: newest stable (falls back to newest of any kind)
   * - latest: newest of any kind
   * - other: the explicitly chosen version
   */
  const resolveLoaderVersion = async (
    loader: LoaderType,
    mcVersion: string,
    mode: LoaderVersionMode,
    explicit?: string,
  ): Promise<string> => {
    if (loader === 'vanilla') return ''
    if (mode === 'other') {
      if (!explicit) throw new Error('no loader version selected')
      return explicit
    }
    const versions = await getLoaderVersions(loader, mcVersion)
    if (!versions.length) throw new Error(`no ${loader} versions for ${mcVersion}`)
    if (mode === 'stable') {
      return (versions.find(v => v.stable) ?? versions[0]).version
    }
    return versions[0].version // latest
  }

  return { getMinecraftVersions, getLoaderVersions, resolveLoaderVersion }
}
