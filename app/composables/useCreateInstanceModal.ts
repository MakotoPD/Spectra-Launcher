/**
 * Shared open/close state for the create-instance modal, so the sidebar button
 * and the globally-mounted <CreateInstanceModal /> can talk to each other.
 */
export const useCreateInstanceModal = () => {
  const isOpen = useState('create-instance-open', () => false)

  const open = () => {
    isOpen.value = true
  }
  const close = () => {
    isOpen.value = false
  }

  return { isOpen, open, close }
}
