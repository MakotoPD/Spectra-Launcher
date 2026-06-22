<template>
  <div class="space-y-4">
    <!-- toolbar -->
    <div class="flex items-center justify-between">
      <p class="text-xs text-muted">
        {{ loading ? $t('common.loading') : $t('content.count', { n: count }) }}
      </p>
      <div class="flex items-center gap-1.5">
        <UButton
          v-if="installKind"
          icon="i-lucide-plus"
          size="xs"
          :label="$t('modrinth.add')"
          @click="openAdd"
        />
        <UButton
          v-if="tab === 'servers'"
          icon="i-lucide-plus"
          size="xs"
          :label="$t('content.addServer')"
          @click="addServerOpen = true"
        />
        <UButton
          icon="i-lucide-refresh-cw"
          color="neutral"
          variant="ghost"
          size="xs"
          :loading="loading"
          :label="$t('content.refresh')"
          @click="load"
        />
      </div>
    </div>

    <p v-if="error" class="text-sm text-error">{{ error }}</p>

    <!-- empty -->
    <div v-else-if="!loading && count === 0" class="flex flex-col items-center justify-center gap-3 py-16 text-center">
      <UIcon :name="emptyIcon" class="size-10 text-neutral-600" />
      <p class="text-sm text-muted">{{ $t('content.empty') }}</p>
    </div>

    <!-- screenshots: image grid -->
    <div
      v-else-if="tab === 'screenshots'"
      class="grid gap-3"
      style="grid-template-columns:repeat(auto-fill,minmax(220px,1fr))"
    >
      <button
        v-for="(s, i) in screenshots"
        :key="s.path"
        type="button"
        class="group overflow-hidden rounded-xl border border-default bg-white/3 text-left transition hover:border-primary-500/40"
        @click="lightboxIndex = i"
      >
        <img :src="assetUrl(s.path)" loading="lazy" class="aspect-video w-full object-cover transition group-hover:opacity-90" :alt="s.name" >
        <div class="truncate px-2.5 py-1.5 text-[11px] text-neutral-400" :title="s.name">{{ s.name }}</div>
      </button>
    </div>

    <!-- worlds -->
    <div v-else-if="tab === 'worlds'" class="space-y-2">
      <div
        v-for="w in worlds"
        :key="w.folder"
        class="flex items-center gap-3 rounded-xl border border-default bg-white/3 p-3"
      >
        <img
          v-if="w.icon_path"
          :src="assetUrl(w.icon_path)"
          class="size-11 shrink-0 rounded-lg object-cover"
          :alt="w.name"
        />
        <div v-else class="flex size-11 shrink-0 items-center justify-center rounded-lg bg-white/5">
          <UIcon name="i-lucide-globe" class="size-5 text-neutral-500" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="truncate font-medium">{{ w.name }}</div>
          <div class="mt-0.5 flex flex-wrap items-center gap-2 text-[11px] text-neutral-400">
            <span v-if="w.version" class="font-mono">{{ w.version }}</span>
            <span v-if="w.game_mode">· {{ $t(`content.gameMode.${w.game_mode}`) }}</span>
            <span v-if="w.last_played">· {{ formatDate(w.last_played) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- resource packs / datapacks -->
    <div v-else-if="tab === 'resourcepacks' || tab === 'datapacks'" class="space-y-2">
      <div
        v-for="p in packs"
        :key="p.filename"
        class="flex items-center gap-3 rounded-xl border border-default bg-white/3 p-3"
      >
        <img
          v-if="p.icon"
          :src="p.icon"
          class="size-11 shrink-0 rounded-lg object-cover [image-rendering:pixelated]"
          :alt="p.name"
        />
        <div v-else class="flex size-11 shrink-0 items-center justify-center rounded-lg bg-white/5">
          <UIcon :name="emptyIcon" class="size-5 text-neutral-500" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span class="truncate font-medium">{{ p.name }}</span>
            <UBadge v-if="p.is_zip" color="neutral" variant="subtle" size="xs" label="zip" />
          </div>
          <div v-if="p.description" class="mt-0.5 truncate text-[12px] text-neutral-400" :title="p.description">
            {{ p.description }}
          </div>
          <div class="mt-0.5 truncate font-mono text-[10px] text-neutral-600" :title="p.filename">{{ p.filename }}</div>
        </div>
        <span v-if="p.pack_format != null" class="shrink-0 font-mono text-[11px] text-neutral-500">
          {{ $t('content.format') }} {{ p.pack_format }}
        </span>
        <UButton
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          size="xs"
          :title="$t('common.remove')"
          @click="removeContent(p.filename)"
        />
      </div>
    </div>

    <!-- shaders -->
    <div v-else-if="tab === 'shaders'" class="space-y-2">
      <div
        v-for="s in shaders"
        :key="s.filename"
        class="flex items-center gap-3 rounded-xl border border-default bg-white/3 p-3"
      >
        <div class="flex size-11 shrink-0 items-center justify-center rounded-lg bg-white/5">
          <UIcon name="i-lucide-sparkles" class="size-5 text-neutral-500" />
        </div>
        <div class="min-w-0 flex-1">
          <span class="block truncate font-medium">{{ s.name }}</span>
          <span class="block truncate font-mono text-[10px] text-neutral-600" :title="s.filename">{{ s.filename }}</span>
        </div>
        <UBadge v-if="s.is_zip" color="neutral" variant="subtle" size="xs" label="zip" />
        <UButton
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          size="xs"
          :title="$t('common.remove')"
          @click="removeContent(s.filename)"
        />
      </div>
    </div>

    <!-- servers -->
    <div v-else-if="tab === 'servers'" class="space-y-2">
      <div
        v-for="(s, i) in servers"
        :key="`${s.ip}-${i}`"
        class="flex items-center gap-3 rounded-xl border border-default bg-white/3 p-3"
        :class="{ 'opacity-55': s.hidden }"
      >
        <img v-if="s.icon" :src="s.icon" class="size-11 shrink-0 rounded-lg object-cover [image-rendering:pixelated]" :alt="s.name" >
        <div v-else class="flex size-11 shrink-0 items-center justify-center rounded-lg bg-white/5">
          <UIcon name="i-lucide-server" class="size-5 text-neutral-500" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="truncate font-medium">{{ s.name || s.ip }}</div>
          <div class="truncate font-mono text-[11px] text-neutral-500">{{ s.ip }}</div>
        </div>
        <UBadge v-if="s.hidden" color="neutral" variant="subtle" size="xs" :label="$t('content.hidden')" />
        <UButton
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          size="xs"
          :title="$t('common.remove')"
          @click="removeServer(i)"
        />
      </div>
    </div>

    <!-- add server modal -->
    <UModal v-model:open="addServerOpen" :title="$t('content.addServer')">
      <template #body>
        <div class="space-y-3">
          <UFormField :label="$t('content.serverName')">
            <UInput v-model="newServer.name" :placeholder="$t('content.serverNamePlaceholder')" class="w-full" autofocus />
          </UFormField>
          <UFormField :label="$t('content.serverAddress')">
            <UInput v-model="newServer.ip" placeholder="mc.example.com" class="w-full" @keydown.enter="addServer" />
          </UFormField>
        </div>
      </template>
      <template #footer>
        <div class="flex w-full justify-end gap-2">
          <UButton variant="ghost" color="neutral" :label="$t('common.cancel')" @click="addServerOpen = false" />
          <UButton :label="$t('common.add')" :loading="addingServer" :disabled="!newServer.ip.trim()" @click="addServer" />
        </div>
      </template>
    </UModal>

    <!-- screenshot lightbox -->
    <UModal v-model:open="lightboxOpen" :ui="{ content: 'max-w-5xl w-[92vw]' }">
      <template #content>
        <div v-if="lightbox" class="flex flex-col">
          <div class="flex items-center justify-between gap-2 border-b border-default px-4 py-2.5">
            <span class="truncate text-sm font-medium" :title="lightbox.name">
              {{ lightbox.name }}
              <span class="ml-1 text-xs text-muted">{{ (lightboxIndex ?? 0) + 1 }} / {{ screenshots.length }}</span>
            </span>
            <div class="flex items-center gap-1.5">
              <UButton icon="i-lucide-download" size="xs" color="neutral" variant="soft" :label="$t('content.download')" @click="downloadShot(lightbox)" />
              <UButton icon="i-lucide-folder-open" size="xs" color="neutral" variant="soft" :label="$t('content.openLocation')" @click="revealShot(lightbox)" />
              <UButton icon="i-lucide-x" size="xs" color="neutral" variant="ghost" square @click="lightbox = null" />
            </div>
          </div>

          <div class="relative bg-black/40">
            <img :src="assetUrl(lightbox.path)" class="max-h-[78vh] w-full object-contain" :alt="lightbox.name" >
            <UButton
              v-if="screenshots.length > 1"
              icon="i-lucide-chevron-left"
              color="neutral"
              class="absolute top-1/2 left-2 -translate-y-1/2 rounded-full opacity-80 hover:opacity-100"
              @click="step(-1)"
            />
            <UButton
              v-if="screenshots.length > 1"
              icon="i-lucide-chevron-right"
              color="neutral"
              class="absolute top-1/2 right-2 -translate-y-1/2 rounded-full opacity-80 hover:opacity-100"
              @click="step(1)"
            />
          </div>
        </div>
      </template>
    </UModal>
  </div>
</template>

<script setup lang="ts">
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { ScreenshotInfo, WorldInfo, PackInfo, ShaderInfo, ServerInfo } from '~/types/launcher'
import type { ContentKind } from '~/types/modrinth'

type ContentTab = 'screenshots' | 'worlds' | 'resourcepacks' | 'datapacks' | 'shaders' | 'servers'

const props = defineProps<{ instanceId: string; tab: ContentTab }>()
const { t } = useI18n()
const browser = useModrinthBrowser()
const instances = useInstancesStore()

const screenshots = ref<ScreenshotInfo[]>([])
const worlds = ref<WorldInfo[]>([])
const packs = ref<PackInfo[]>([])
const shaders = ref<ShaderInfo[]>([])
const servers = ref<ServerInfo[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const COMMANDS: Record<ContentTab, string> = {
  screenshots: 'list_screenshots',
  worlds: 'list_worlds',
  resourcepacks: 'list_resource_packs',
  datapacks: 'list_data_packs',
  shaders: 'list_shaders',
  servers: 'list_servers',
}

const EMPTY_ICONS: Record<ContentTab, string> = {
  screenshots: 'i-lucide-camera',
  worlds: 'i-lucide-globe',
  resourcepacks: 'i-lucide-image',
  datapacks: 'i-lucide-package',
  shaders: 'i-lucide-sparkles',
  servers: 'i-lucide-server',
}
const emptyIcon = computed(() => EMPTY_ICONS[props.tab])

const count = computed(() => {
  switch (props.tab) {
    case 'screenshots': return screenshots.value.length
    case 'worlds': return worlds.value.length
    case 'resourcepacks':
    case 'datapacks': return packs.value.length
    case 'shaders': return shaders.value.length
    case 'servers': return servers.value.length
    default: return 0
  }
})

async function load() {
  loading.value = true
  error.value = null
  try {
    const result = await invoke<unknown>(COMMANDS[props.tab], { id: props.instanceId })
    switch (props.tab) {
      case 'screenshots': screenshots.value = result as ScreenshotInfo[]; break
      case 'worlds': worlds.value = result as WorldInfo[]; break
      case 'resourcepacks':
      case 'datapacks': packs.value = result as PackInfo[]; break
      case 'shaders': shaders.value = result as ShaderInfo[]; break
      case 'servers': servers.value = result as ServerInfo[]; break
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

const assetUrl = (path: string) => convertFileSrc(path)
const formatDate = (ms: number) => new Date(ms).toLocaleDateString()

// --- servers: add / remove ---
const toast = useToast()
const addServerOpen = ref(false)
const addingServer = ref(false)
const newServer = reactive({ name: '', ip: '' })

async function addServer() {
  if (!newServer.ip.trim()) return
  addingServer.value = true
  try {
    await invoke('add_server', { id: props.instanceId, name: newServer.name.trim(), ip: newServer.ip.trim() })
    addServerOpen.value = false
    newServer.name = ''
    newServer.ip = ''
    await load()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  } finally {
    addingServer.value = false
  }
}

async function removeServer(index: number) {
  try {
    await invoke('delete_server', { id: props.instanceId, index })
    await load()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
}

// Delete a resource pack / shader / datapack (kind = current tab's install kind).
async function removeContent(filename: string) {
  if (!installKind.value) return
  try {
    await invoke('delete_content', { id: props.instanceId, kind: installKind.value, filename })
    await load()
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
}

// --- screenshots: lightbox / navigation / download / reveal ---
const lightboxIndex = ref<number | null>(null)
const lightbox = computed<ScreenshotInfo | null>({
  get: () => (lightboxIndex.value !== null ? screenshots.value[lightboxIndex.value] ?? null : null),
  set: (v) => { if (v === null) lightboxIndex.value = null },
})
const lightboxOpen = computed({
  get: () => lightboxIndex.value !== null,
  set: (v: boolean) => { if (!v) lightboxIndex.value = null },
})

/** Moves through the gallery, wrapping around. */
function step(delta: number) {
  if (lightboxIndex.value === null || !screenshots.value.length) return
  const n = screenshots.value.length
  lightboxIndex.value = (lightboxIndex.value + delta + n) % n
}

function onKey(e: KeyboardEvent) {
  if (lightboxIndex.value === null) return
  if (e.key === 'ArrowRight') { e.preventDefault(); step(1) }
  else if (e.key === 'ArrowLeft') { e.preventDefault(); step(-1) }
  else if (e.key === 'Escape') lightboxIndex.value = null
}
onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

async function downloadShot(s: ScreenshotInfo) {
  try {
    const dest = await save({ defaultPath: s.name, filters: [{ name: 'PNG', extensions: ['png'] }] })
    if (!dest) return
    await invoke('copy_file', { from: s.path, to: dest })
    toast.add({ title: t('content.downloaded'), color: 'success' })
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
}

async function revealShot(s: ScreenshotInfo) {
  try {
    await invoke('reveal_in_explorer', { path: s.path })
  } catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
}

// Tabs whose content can be downloaded from Modrinth.
const INSTALL_KINDS: Partial<Record<ContentTab, ContentKind>> = {
  shaders: 'shader',
  datapacks: 'datapack',
  resourcepacks: 'resourcepack',
}
const installKind = computed(() => INSTALL_KINDS[props.tab] ?? null)

function openAdd() {
  if (!installKind.value) return
  const instance = instances.instances.find(i => i.id === props.instanceId)
  browser.open({
    kind: installKind.value,
    mode: 'install',
    instanceId: props.instanceId,
    gameVersion: instance?.mc_version,
    loader: instance?.loader.type,
    onInstalled: () => load(),
  })
}

watch(() => [props.instanceId, props.tab], load, { immediate: true })
</script>
