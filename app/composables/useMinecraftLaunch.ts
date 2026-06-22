import { invoke } from '@tauri-apps/api/core'
import { toValue, type MaybeRefOrGetter } from 'vue'

/**
 * Drives launching an instance and exposes its live state. Event listening +
 * activity tracking live in `useActivityCenter` (a single global hub); this
 * composable derives a per-instance `stage`/`progress`/`log`/`error` view.
 *
 * Pass the instance id you're interested in so the state is scoped to it (e.g.
 * the instance page). Without an id, `stage`/`progress` fall back to the
 * highest-priority activity (used by views that only call `launch`).
 */
export const useMinecraftLaunch = (instanceId?: MaybeRefOrGetter<string | undefined>) => {
  const ac = useActivityCenter()

  // Per-instance state, keyed by id, so concurrent instances don't bleed into
  // each other's UI.
  const launchingIds = useState<Record<string, boolean>>('mc-launching-ids', () => ({}))
  const errors = useState<Record<string, string | null>>('mc-errors', () => ({}))

  const id = computed(() => toValue(instanceId))

  const activity = computed(() => (id.value ? ac.activityFor(id.value).value : ac.top.value))

  const stage = computed<'idle' | 'installing' | 'running'>(() => {
    const a = activity.value
    if (!a) return 'idle'
    return a.kind === 'install' ? 'installing' : 'running'
  })

  const progress = computed(() => {
    const a = activity.value
    return a && a.kind === 'install'
      ? { current: a.current, total: a.total }
      : { current: 0, total: 0 }
  })

  const log = computed(() => (id.value ? ac.logsFor(id.value).value : []))
  const error = computed(() => (id.value ? errors.value[id.value] ?? null : null))
  const launching = computed(() => (id.value ? !!launchingIds.value[id.value] : Object.values(launchingIds.value).some(Boolean)))

  const runningId = computed(() => ac.list.value.find(a => a.kind === 'running')?.instanceId ?? null)

  const launch = async (launchId: string) => {
    errors.value = { ...errors.value, [launchId]: null }
    ac.clearLog(launchId)
    launchingIds.value = { ...launchingIds.value, [launchId]: true }
    await ac.attach()
    try {
      await invoke('launch_instance', { id: launchId })
    } catch (e) {
      errors.value = { ...errors.value, [launchId]: String(e) }
      ac.clear(launchId)
      throw e
    } finally {
      launchingIds.value = { ...launchingIds.value, [launchId]: false }
    }
  }

  return {
    launching,
    runningId,
    stage,
    progress,
    log,
    error,
    launch,
    attach: ac.attach,
    detach: ac.detach,
  }
}
