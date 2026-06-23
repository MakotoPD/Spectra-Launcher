// Callback kept module-level (SPA only) so it survives without serialization.
let resolvedCb: (() => void) | null = null

/**
 * Global state for the "blocked mods" resolver — CurseForge files whose authors
 * disallow third-party download. The user grabs them manually; we watch a folder
 * and copy matches into the instance.
 */
export const useBlockedModsModal = () => {
  const isOpen = useState('blocked-open', () => false)
  const instanceId = useState<string | null>('blocked-instance', () => null)

  const open = (id: string, onResolved?: () => void) => {
    instanceId.value = id
    resolvedCb = onResolved ?? null
    isOpen.value = true
  }
  const close = () => {
    isOpen.value = false
  }
  const notifyResolved = () => resolvedCb?.()

  return { isOpen, instanceId, open, close, notifyResolved }
}
