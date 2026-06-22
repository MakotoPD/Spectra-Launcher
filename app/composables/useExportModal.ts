/**
 * Global open/close state for the shared <ExportInstanceModal>. Open it from
 * anywhere (instance page, home context menu) with the instance to export.
 */
export const useExportModal = () => {
  const isOpen = useState('export-modal-open', () => false)
  const target = useState<{ id: string, name: string } | null>('export-modal-target', () => null)

  const open = (id: string, name: string) => {
    target.value = { id, name }
    isOpen.value = true
  }
  const close = () => {
    isOpen.value = false
  }

  return { isOpen, target, open, close }
}
