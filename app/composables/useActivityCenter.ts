import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { MultiProgress, ConsoleLine, ExitInfo } from '~/types/launcher'

/** A single thing happening for an instance, surfaced in the titlebar. */
export interface Activity {
  instanceId: string
  kind: 'install' | 'running'
  current: number
  total: number
}

// Higher number = higher priority. The titlebar shows the activity with the
// highest priority across all instances.
const PRIORITY: Record<Activity['kind'], number> = {
  install: 2,
  running: 1,
}

// Module-level so the listeners are attached exactly once for the whole app,
// no matter how many components/composables call `attach()` (even concurrently).
let unlisteners: UnlistenFn[] = []
let attachPromise: Promise<void> | null = null

/**
 * Single global hub for the live launch/install events streamed from Rust.
 * Owns the Tauri listeners (attached once) and exposes the current activities
 * keyed by instance id, plus the highest-priority one for the titlebar.
 */
export const useActivityCenter = () => {
  const activities = useState<Record<string, Activity>>('mc-activities', () => ({}))
  // Console output is kept per instance so each instance page shows only its own.
  const logs = useState<Record<string, string[]>>('mc-logs', () => ({}))
  // Ad-hoc frontend-driven operations (mod/modpack install/update, …) → label.
  const tasks = useState<Record<string, string>>('mc-tasks', () => ({}))

  /** Registers a running operation in the titlebar; returns its id for endTask. */
  function startTask(label: string): string {
    const id = crypto.randomUUID()
    tasks.value = { ...tasks.value, [id]: label }
    return id
  }
  function endTask(id: string) {
    if (!tasks.value[id]) return
    const next = { ...tasks.value }
    delete next[id]
    tasks.value = next
  }
  /** Runs `fn` while showing `label` in the titlebar. */
  async function withTask<T>(label: string, fn: () => Promise<T>): Promise<T> {
    const id = startTask(label)
    try {
      return await fn()
    } finally {
      endTask(id)
    }
  }

  const upsert = (id: string, patch: Partial<Activity> & Pick<Activity, 'kind'>) => {
    const prev = activities.value[id]
    activities.value = {
      ...activities.value,
      [id]: { instanceId: id, current: 0, total: 0, ...prev, ...patch },
    }
  }

  const clear = (id: string) => {
    if (!activities.value[id]) return
    const next = { ...activities.value }
    delete next[id]
    activities.value = next
  }

  const attach = () => {
    if (!attachPromise) {
      attachPromise = (async () => {
        unlisteners.push(
          await listen<MultiProgress>('mc://multi-progress', (e) => {
            upsert(e.payload.instance_id, { kind: 'install', current: e.payload.current, total: e.payload.total })
          }),
          await listen<ConsoleLine>('mc://console', (e) => {
            const iid = e.payload.instance_id
            upsert(iid, { kind: 'running' })
            const buf = logs.value[iid] ?? []
            buf.push(e.payload.line)
            if (buf.length > 2000) buf.splice(0, buf.length - 2000)
            logs.value = { ...logs.value, [iid]: buf }
          }),
          await listen<ExitInfo>('mc://exited', (e) => {
            clear(e.payload.instance_id)
          }),
        )
      })()
    }
    return attachPromise
  }

  const detach = () => {
    unlisteners.forEach(u => u())
    unlisteners = []
    attachPromise = null
  }

  const list = computed(() => Object.values(activities.value))

  /** Highest-priority activity across all instances, or null when idle. */
  const top = computed<Activity | null>(() => {
    let best: Activity | null = null
    for (const a of list.value) {
      if (!best || PRIORITY[a.kind] > PRIORITY[best.kind]) best = a
    }
    return best
  })

  /** Marks an instance as running without a console event — used to reflect
   *  instances adopted from a previous launcher session after a restart. */
  const markRunning = (id: string) => {
    if (activities.value[id]) return
    upsert(id, { kind: 'running' })
  }

  const activityFor = (id: string) => computed(() => activities.value[id] ?? null)
  const logsFor = (id: string) => computed(() => logs.value[id] ?? [])
  const clearLog = (id: string) => {
    logs.value = { ...logs.value, [id]: [] }
  }

  const taskLabels = computed(() => Object.values(tasks.value))

  // Live-logs modal (opened by clicking the titlebar activity).
  const liveLogsOpen = useState('mc-livelogs-open', () => false)
  const liveLogsInstance = useState<string | null>('mc-livelogs-instance', () => null)
  function openLiveLogs(instanceId?: string) {
    liveLogsInstance.value = instanceId ?? top.value?.instanceId ?? null
    liveLogsOpen.value = true
  }

  return { activities, logs, tasks, taskLabels, startTask, endTask, withTask, attach, detach, list, top, activityFor, logsFor, clear, clearLog, markRunning, liveLogsOpen, liveLogsInstance, openLiveLogs }
}
