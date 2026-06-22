export interface LinkModsRequest {
  instanceId: string
  /** Base filenames of local mods to try to link. */
  files: string[]
  cfEnabled: boolean
  /** Called after the whole flow finishes (matched or skipped). */
  onDone?: () => void
}

/**
 * Global state for the per-file "link mods" dialog (provider choice), opened
 * from the mods tab. Mirrors PrismLauncher's ChooseProviderDialog flow.
 */
export const useLinkModsModal = () => {
  const isOpen = useState('linkmods-open', () => false)
  const req = useState<LinkModsRequest | null>('linkmods-req', () => null)

  const open = (r: LinkModsRequest) => {
    req.value = r
    isOpen.value = true
  }
  const close = () => {
    isOpen.value = false
  }

  return { isOpen, req, open, close }
}
