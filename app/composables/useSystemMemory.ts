import { invoke } from '@tauri-apps/api/core'

/** Total system RAM (MB) and the allocatable max (90% of it), for memory sliders. */
export const useSystemMemory = () => {
  const totalMb = useState('system-memory-mb', () => 0)

  const ensure = async () => {
    if (totalMb.value) return
    try {
      totalMb.value = await invoke<number>('get_system_memory_mb')
    } catch {
      totalMb.value = 0
    }
  }

  const MIN_MB = 512
  // Cap at 90% of total RAM; fall back to 16 GB until detected.
  const maxMb = computed(() => (totalMb.value ? Math.max(MIN_MB, Math.floor((totalMb.value * 0.9) / 256) * 256) : 16384))

  return { totalMb, maxMb, minMb: MIN_MB, ensure }
}
