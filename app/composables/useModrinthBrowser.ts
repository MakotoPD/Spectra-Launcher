import type { LoaderType, Instance } from '~/types/launcher'
import type { ContentKind } from '~/types/modrinth'

export interface ModrinthBrowserConfig {
  /** What we're browsing for. */
  kind: ContentKind
  /** 'install' downloads into `instanceId`; 'createModpack' makes a new instance. */
  mode: 'install' | 'createModpack'
  instanceId?: string
  /** Pre-applied filters (e.g. derived from the target instance). */
  gameVersion?: string
  loader?: LoaderType
  /** Pre-fills the search box (e.g. to focus a specific project). */
  query?: string
  /** Called after a successful install (with the new instance for modpacks). */
  onInstalled?: (instance?: Instance) => void
}

/**
 * Global open/close state for the shared <ModrinthBrowser> modal. Open it from
 * anywhere with a config describing what to browse and where to install.
 */
export const useModrinthBrowser = () => {
  const isOpen = useState('modrinth-browser-open', () => false)
  const config = useState<ModrinthBrowserConfig | null>('modrinth-browser-config', () => null)

  const open = (cfg: ModrinthBrowserConfig) => {
    config.value = cfg
    isOpen.value = true
  }
  const close = () => {
    isOpen.value = false
  }

  return { isOpen, config, open, close }
}
