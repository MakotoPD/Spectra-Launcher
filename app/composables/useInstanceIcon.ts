import { invoke, convertFileSrc } from '@tauri-apps/api/core'

// Resolved asset-protocol URLs for instance icons, cached by id so the sidebar
// list doesn't re-resolve on every render.
const cache = new Map<string, string>()

/**
 * Returns an `asset://`-style URL for an instance's `icon.png`, or null if it
 * has no icon. `hasIcon` is `instance.icon` truthiness (the "icon.png" marker).
 */
export async function resolveInstanceIcon(id: string, hasIcon: boolean): Promise<string | null> {
  if (!hasIcon) return null
  const cached = cache.get(id)
  if (cached) return cached
  const path = await invoke<string | null>('get_instance_icon_path', { id })
  if (!path) return null
  const url = convertFileSrc(path)
  cache.set(id, url)
  return url
}

/** Drop a cached icon URL (call if the icon changes or the instance is removed). */
export function invalidateInstanceIcon(id: string) {
  cache.delete(id)
}
