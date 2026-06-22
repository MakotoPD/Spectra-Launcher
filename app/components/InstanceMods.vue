<template>
  <div class="space-y-4">
    <!-- toolbar -->
    <div class="flex flex-wrap items-center gap-2">
      <UInput
        v-model="search"
        icon="i-lucide-search"
        variant="soft"
        :placeholder="$t('mods.searchPlaceholder')"
        class="min-w-44 flex-1"
      />
      <USelect v-model="perPage" :items="perPageItems" value-key="value" class="w-28" />
      <UButton
        v-if="updateCount"
        icon="i-lucide-circle-arrow-up"
        color="primary"
        size="sm"
        :loading="updatingAll"
        :label="$t('mods.updateAll', { n: updateCount })"
        @click="updateAll"
      />
      <UButton
        v-if="hasLocal"
        icon="i-lucide-link"
        color="neutral"
        variant="soft"
        size="sm"
        :loading="linking"
        :label="$t('mods.linkModrinth')"
        title="Try to link local mods to Modrinth"
        @click="linkLocal"
      />
      <UButton icon="i-lucide-package-search" size="sm" :label="$t('modrinth.add')" @click="openBrowser" />
      <UButton
        icon="i-lucide-refresh-cw"
        color="neutral"
        variant="ghost"
        size="sm"
        :loading="loading"
        :title="$t('content.refresh')"
        square
        @click="load"
      />
    </div>

    <p v-if="error" class="text-sm text-error">{{ error }}</p>

    <!-- empty: no mods at all -->
    <div v-else-if="!loading && !mods.length" class="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <UIcon name="i-lucide-blocks" class="size-10 text-neutral-600" />
      <p class="max-w-sm text-sm text-muted">{{ $t('instance.modsHint') }}</p>
      <UButton icon="i-lucide-package-search" :label="$t('modrinth.browseMods')" @click="openBrowser" />
    </div>

    <!-- no search results -->
    <div v-else-if="!filtered.length" class="py-12 text-center text-sm text-muted">{{ $t('content.empty') }}</div>

    <!-- table -->
    <template v-else>
      <p class="text-xs text-muted">{{ $t('content.count', { n: filtered.length }) }}</p>

      <div class="overflow-hidden rounded-xl border border-default">
        <!-- header (click to sort: asc → desc → off) -->
        <div class="grid grid-cols-[auto_2.75rem_minmax(0,1fr)_8rem_7rem_4rem_auto] items-center gap-3 border-b border-default bg-white/4 px-3 py-2 text-[11px] font-medium tracking-wide text-neutral-500 uppercase">
          <span />
          <button type="button" class="flex items-center justify-center gap-1 transition hover:text-neutral-200" :class="{ 'text-primary-400': sortCol === 'state' }" @click="toggleSort('state')">
            {{ $t('mods.col.enabled') }}
            <UIcon :name="sortIcon('state')" class="size-3" :class="{ 'opacity-30': sortCol !== 'state' }" />
          </button>
          <button type="button" class="flex items-center gap-1 transition hover:text-neutral-200" :class="{ 'text-primary-400': sortCol === 'name' }" @click="toggleSort('name')">
            {{ $t('mods.col.name') }}
            <UIcon :name="sortIcon('name')" class="size-3" :class="{ 'opacity-30': sortCol !== 'name' }" />
          </button>
          <button type="button" class="flex items-center gap-1 transition hover:text-neutral-200" :class="{ 'text-primary-400': sortCol === 'version' }" @click="toggleSort('version')">
            {{ $t('mods.col.version') }}
            <UIcon :name="sortIcon('version')" class="size-3" :class="{ 'opacity-30': sortCol !== 'version' }" />
          </button>
          <button type="button" class="flex items-center gap-1 transition hover:text-neutral-200" :class="{ 'text-primary-400': sortCol === 'updated' }" @click="toggleSort('updated')">
            {{ $t('mods.col.updated') }}
            <UIcon :name="sortIcon('updated')" class="size-3" :class="{ 'opacity-30': sortCol !== 'updated' }" />
          </button>
          <span />
          <span />
        </div>

        <!-- rows -->
        <div
          v-for="m in paged"
          :key="m.filename"
          class="grid grid-cols-[auto_2.75rem_minmax(0,1fr)_8rem_7rem_4rem_auto] items-center gap-3 border-b border-default/50 px-3 py-2 transition last:border-0 hover:bg-white/3"
          :class="{ 'opacity-55': !m.enabled }"
        >
          <!-- toggle -->
          <div class="flex justify-center">
            <USwitch
              :model-value="m.enabled"
              :title="m.enabled ? $t('mods.disable') : $t('mods.enable')"
              @update:model-value="toggle(m, $event)"
            />
          </div>
          <!-- icon -->
          <img v-if="m.icon_url" :src="m.icon_url" class="size-9 rounded-lg object-cover" :alt="m.name ?? m.filename" />
          <div v-else class="flex size-9 items-center justify-center rounded-lg bg-white/5">
            <UIcon name="i-lucide-blocks" class="size-4.5 text-neutral-500" />
          </div>

          <!-- name + provider -->
          <div class="min-w-0">
            <div class="group/n flex items-center gap-1">
              <span class="truncate font-medium" :title="m.filename">{{ m.name ?? m.filename }}</span>
              <UButton
                v-if="m.project_id"
                icon="i-lucide-external-link"
                color="neutral"
                variant="ghost"
                size="xs"
                class="shrink-0 opacity-0 transition group-hover/n:opacity-100"
                :title="$t('mods.openPage')"
                @click="openModPage(m)"
              />
            </div>
            <UBadge color="neutral" variant="subtle" size="xs" class="mt-0.5" :label="$t(`mods.provider.${m.provider}`)" />
          </div>

          <!-- version -->
          <span class="truncate font-mono text-xs text-neutral-400" :title="m.version ?? ''">{{ m.version ?? '—' }}</span>

          <!-- updated -->
          <span class="text-xs text-neutral-500">{{ formatDate(m.modified) }}</span>



          <!-- actions -->
          <div class="flex items-center justify-end gap-0.5">
            <UButton
              v-if="!m.project_id"
              icon="i-lucide-search"
              color="neutral"
              variant="ghost"
              size="xs"
              :loading="linking"
              :title="$t('mods.checkModrinth')"
              @click="linkLocal"
            />
            <UButton
              v-if="m.project_id && updates[m.project_id]"
              icon="i-lucide-circle-arrow-up"
              color="primary"
              variant="soft"
              size="xs"
              :loading="updatingId === m.filename"
              :title="$t('mods.updateTo', { v: updates[m.project_id]?.version_number ?? '' })"
              @click="updateMod(m)"
            />
            <UButton
              v-if="m.project_id"
              icon="i-lucide-replace"
              color="neutral"
              variant="ghost"
              size="xs"
              :title="$t('mods.changeVersion')"
              @click="openVersions(m)"
            />
            <UButton
              icon="i-lucide-trash-2"
              color="error"
              variant="ghost"
              size="xs"
              :title="$t('common.remove')"
              @click="remove(m)"
            />
          </div>
        </div>
      </div>

      <!-- pagination -->
      <div v-if="filtered.length > perPage" class="flex justify-center pt-1">
        <UPagination v-model:page="page" :total="filtered.length" :items-per-page="perPage" />
      </div>
    </template>

    <!-- change-version modal -->
    <UModal v-model:open="versionsOpen" :title="$t('mods.pickVersion')" :ui="{ content: 'max-w-lg' }">
      <template #body>
        <div v-if="loadingVersions" class="py-8 text-center text-sm text-muted">{{ $t('common.loading') }}</div>
        <div v-else-if="!modVersions.length" class="py-8 text-center text-sm text-muted">{{ $t('modrinth.noVersions') }}</div>
        <div v-else class="max-h-[60vh] space-y-1.5 overflow-y-auto">
          <div
            v-for="v in modVersions"
            :key="v.id"
            class="flex items-center gap-3 rounded-lg border border-default bg-white/3 p-2.5"
          >
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="truncate text-sm font-medium">{{ v.name || v.version_number }}</span>
                <UBadge v-if="versionsMod && installedVersionId === v.id" color="success" variant="subtle" size="xs" :label="$t('mods.installed')" />
              </div>
              <div class="mt-0.5 truncate text-[11px] text-neutral-500">
                {{ v.game_versions.slice(0, 4).join(', ') }}<span v-if="v.loaders.length"> · {{ v.loaders.join(', ') }}</span>
              </div>
            </div>
            <UButton
              size="xs"
              color="neutral"
              variant="soft"
              :loading="installingVersionId === v.id"
              :disabled="installedVersionId === v.id || !!installingVersionId"
              :label="$t('mods.useVersion')"
              @click="chooseVersion(v.id)"
            />
          </div>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import type { ModEntry, Instance } from '~/types/launcher'
import type { ModUpdate, ModrinthVersion } from '~/types/modrinth'

const props = defineProps<{ instanceId: string }>()
const toast = useToast()
const { t } = useI18n()
const browser = useModrinthBrowser()
const instances = useInstancesStore()
const modrinth = useModrinth()
const activity = useActivityCenter()

const mods = ref<ModEntry[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// Target instance filters (loader + game version) for Modrinth lookups.
const instance = computed(() => instances.instances.find((i: Instance) => i.id === props.instanceId))
const filterLoaders = computed(() => {
  const lt = instance.value?.loader.type
  return lt && lt !== 'vanilla' ? [lt] : undefined
})
const filterGv = computed(() => (instance.value ? [instance.value.mc_version] : undefined))

// --- available updates ---
const updates = ref<Record<string, ModUpdate>>({})
const updatingId = ref<string | null>(null) // filename being updated
const updatingAll = ref(false)
const updateCount = computed(() => mods.value.filter(m => m.project_id && updates.value[m.project_id]).length)

async function refreshUpdates() {
  try {
    const list = await modrinth.checkUpdates(props.instanceId, filterLoaders.value, filterGv.value)
    updates.value = Object.fromEntries(list.map(u => [u.project_id, u]))
  } catch {
    updates.value = {}
  }
}

/** Installs `versionId` for a mod and removes its previous jar. */
async function applyVersion(mod: ModEntry, versionId: string) {
  const added = await modrinth.installWithDeps(props.instanceId, versionId, filterGv.value?.[0], filterLoaders.value?.[0])
  const fresh = added.find(a => a.project_id === mod.project_id)
  if (fresh && fresh.filename !== mod.filename) {
    await invoke('delete_mod', { instanceId: props.instanceId, filename: mod.filename })
  }
}

async function updateMod(mod: ModEntry) {
  const u = mod.project_id ? updates.value[mod.project_id] : null
  if (!u) return
  updatingId.value = mod.filename
  const tid = activity.startTask(t('activity.updatingMod', { name: mod.name ?? mod.filename }))
  try {
    await applyVersion(mod, u.version_id)
    await load()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  } finally {
    activity.endTask(tid)
    updatingId.value = null
  }
}

async function updateAll() {
  updatingAll.value = true
  const tid = activity.startTask(t('activity.updatingMods'))
  try {
    for (const m of mods.value) {
      const pid = m.project_id
      if (!pid) continue
      const u = updates.value[pid]
      if (u) await applyVersion(m, u.version_id)
    }
    await load()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  } finally {
    activity.endTask(tid)
    updatingAll.value = false
  }
}

// --- change version modal ---
const versionsMod = ref<ModEntry | null>(null)
const versionsOpen = computed({
  get: () => versionsMod.value !== null,
  set: (v: boolean) => { if (!v) versionsMod.value = null },
})
const modVersions = ref<ModrinthVersion[]>([])
const loadingVersions = ref(false)
const installingVersionId = ref<string | null>(null)
const installedVersionId = computed(() => versionsMod.value?.version_id ?? null)

async function openVersions(mod: ModEntry) {
  if (!mod.project_id) return
  versionsMod.value = mod
  modVersions.value = []
  loadingVersions.value = true
  try {
    modVersions.value = await modrinth.versions(mod.project_id, filterLoaders.value, filterGv.value)
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  } finally {
    loadingVersions.value = false
  }
}

async function chooseVersion(versionId: string) {
  const mod = versionsMod.value
  if (!mod) return
  installingVersionId.value = versionId
  const tid = activity.startTask(t('activity.changingVersion', { name: mod.name ?? mod.filename }))
  try {
    await applyVersion(mod, versionId)
    versionsMod.value = null
    await load()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  } finally {
    activity.endTask(tid)
    installingVersionId.value = null
  }
}

/** Opens a mod's Modrinth page in the external browser. */
function openModPage(mod: ModEntry) {
  if (mod.project_id) openUrl(`https://modrinth.com/mod/${mod.project_id}`)
}

// --- controls ---
type SortCol = 'name' | 'version' | 'updated' | 'state'
const search = ref('')
const perPage = ref(25)
const page = ref(1)

// Column-header sorting: click cycles asc → desc → off.
const sortCol = ref<SortCol | null>(null)
const sortDir = ref<'asc' | 'desc'>('asc')

function toggleSort(col: SortCol) {
  if (sortCol.value !== col) {
    sortCol.value = col
    sortDir.value = 'asc'
  } else if (sortDir.value === 'asc') {
    sortDir.value = 'desc'
  } else {
    sortCol.value = null // off
  }
}
const sortIcon = (col: SortCol) =>
  sortCol.value === col ? (sortDir.value === 'asc' ? 'i-lucide-arrow-up' : 'i-lucide-arrow-down') : 'i-lucide-chevrons-up-down'

const perPageItems = [10, 25, 50, 100].map(n => ({ label: t('mods.perPage', { n }), value: n }))

const nameOf = (m: ModEntry) => (m.name ?? m.filename).toLowerCase()
const formatDate = (ms: number) => (ms ? new Date(ms).toLocaleDateString() : '—')

// Ascending comparator per column; direction is applied afterwards.
function compare(a: ModEntry, b: ModEntry, col: SortCol): number {
  switch (col) {
    case 'name': return nameOf(a).localeCompare(nameOf(b))
    case 'version': return (a.version ?? '').localeCompare(b.version ?? '', undefined, { numeric: true })
    case 'updated': return a.modified - b.modified
    case 'state': return Number(b.enabled) - Number(a.enabled) // enabled first
  }
}

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  let list = mods.value
  if (q) list = list.filter(m => nameOf(m).includes(q) || m.filename.toLowerCase().includes(q))

  const col = sortCol.value
  if (!col) return list
  const dir = sortDir.value === 'asc' ? 1 : -1
  return [...list].sort((a, b) => compare(a, b, col) * dir || nameOf(a).localeCompare(nameOf(b)))
})

const paged = computed(() => {
  const start = (page.value - 1) * perPage.value
  return filtered.value.slice(start, start + perPage.value)
})

// Reset to page 1 when the view changes; clamp if the list shrinks.
watch([search, sortCol, sortDir, perPage], () => { page.value = 1 })
watch(filtered, () => {
  const max = Math.max(1, Math.ceil(filtered.value.length / perPage.value))
  if (page.value > max) page.value = max
})

async function load() {
  loading.value = true
  error.value = null
  try {
    mods.value = await invoke<ModEntry[]>('list_mods', { instanceId: props.instanceId })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
  void linkAndCheck()
}

// Link manually-added jars to Modrinth (by hash), then check for updates.
async function linkAndCheck() {
  try {
    const matched = await modrinth.matchLocal(props.instanceId)
    if (matched > 0) {
      mods.value = await invoke<ModEntry[]>('list_mods', { instanceId: props.instanceId })
    }
  } catch { /* offline / not found — ignore on auto-run */ }
  refreshUpdates()
}

const hasLocal = computed(() => mods.value.some(m => !m.project_id))
const linking = ref(false)

/** Manual: link local jars to Modrinth by hash, with feedback. */
async function linkLocal() {
  linking.value = true
  const tid = activity.startTask(t('activity.linking'))
  try {
    const matched = await modrinth.matchLocal(props.instanceId)
    mods.value = await invoke<ModEntry[]>('list_mods', { instanceId: props.instanceId })
    toast.add({
      title: matched > 0 ? t('mods.linked', { n: matched }) : t('mods.noMatch'),
      color: matched > 0 ? 'success' : 'neutral',
    })
    refreshUpdates()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  } finally {
    activity.endTask(tid)
    linking.value = false
  }
}

async function toggle(mod: ModEntry, enabled: boolean) {
  try {
    await invoke('set_mod_enabled', { instanceId: props.instanceId, filename: mod.filename, enabled })
    mod.enabled = enabled
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
}

async function remove(mod: ModEntry) {
  try {
    await invoke('delete_mod', { instanceId: props.instanceId, filename: mod.filename })
    mods.value = mods.value.filter(m => m.filename !== mod.filename)
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
}

function openBrowser() {
  const instance = instances.instances.find((i: Instance) => i.id === props.instanceId)
  browser.open({
    kind: 'mod',
    mode: 'install',
    instanceId: props.instanceId,
    gameVersion: instance?.mc_version,
    loader: instance?.loader.type,
    onInstalled: () => load(),
  })
}

watch(() => props.instanceId, load, { immediate: true })
</script>
