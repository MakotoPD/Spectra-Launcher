<template>
  <UModal v-model:open="isOpen" :title="$t('export.title', { name: target?.name ?? '' })" :ui="{ content: 'max-w-xl' }">
    <template #body>
      <div class="space-y-5">
        <!-- format -->
        <div>
          <p class="mb-2 text-sm font-medium">{{ $t('export.format') }}</p>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="f in formats"
              :key="f.value"
              type="button"
              :disabled="f.disabled"
              class="rounded-lg border px-3 py-2.5 text-left transition disabled:cursor-not-allowed disabled:opacity-50"
              :class="format === f.value ? 'border-primary-500 bg-primary-500/10' : 'border-default hover:border-neutral-500'"
              @click="format = f.value"
            >
              <div class="flex items-center gap-1.5 text-sm font-medium">
                <UIcon :name="f.icon" class="size-4" />
                {{ f.label }}
              </div>
              <div class="mt-0.5 text-xs text-muted">{{ f.desc }}</div>
            </button>
          </div>
        </div>

        <!-- mrpack metadata -->
        <div v-if="format === 'mrpack'" class="grid grid-cols-2 gap-3">
          <UFormField :label="$t('export.version')">
            <UInput v-model="version" placeholder="1.0.0" class="w-full" />
          </UFormField>
          <UFormField :label="$t('export.summary')">
            <UInput v-model="summary" :placeholder="$t('export.summaryPlaceholder')" class="w-full" />
          </UFormField>
        </div>

        <!-- file selection -->
        <div>
          <div class="mb-2 flex items-center justify-between">
            <p class="text-sm font-medium">{{ $t('export.include') }}</p>
            <div class="flex items-center gap-2 text-xs">
              <button type="button" class="text-primary-400 hover:underline" @click="selectAll(true)">{{ $t('export.all') }}</button>
              <span class="text-neutral-600">·</span>
              <button type="button" class="text-primary-400 hover:underline" @click="selectAll(false)">{{ $t('export.none') }}</button>
            </div>
          </div>

          <div v-if="loading" class="py-6 text-center text-sm text-muted">{{ $t('common.loading') }}</div>
          <div v-else-if="!entries.length" class="py-6 text-center text-sm text-muted">{{ $t('export.empty') }}</div>
          <div v-else class="max-h-64 space-y-0.5 overflow-y-auto rounded-lg border border-default p-1.5">
            <label
              v-for="e in entries"
              :key="e.name"
              class="flex cursor-pointer items-center gap-2.5 rounded-md px-2 py-1.5 hover:bg-white/5"
            >
              <UCheckbox :model-value="selected.has(e.name)" @update:model-value="toggle(e.name)" />
              <UIcon :name="e.is_dir ? 'i-lucide-folder' : 'i-lucide-file'" class="size-4 shrink-0 text-neutral-500" />
              <span class="min-w-0 flex-1 truncate text-sm">{{ e.name }}</span>
              <span class="shrink-0 font-mono text-[11px] text-muted">{{ fmtSize(e.size) }}</span>
            </label>
          </div>
          <p v-if="format === 'mrpack'" class="mt-2 text-xs text-muted">{{ $t('export.mrpackHint') }}</p>
        </div>

        <p v-if="error" class="text-sm text-error">{{ error }}</p>
      </div>
    </template>

    <template #footer>
      <div class="flex w-full justify-end gap-2">
        <UButton variant="ghost" color="neutral" :label="$t('common.cancel')" @click="close" />
        <UButton
          icon="i-lucide-package"
          :label="$t('export.exportBtn')"
          :loading="exporting"
          :disabled="!selected.size"
          @click="doExport"
        />
      </div>
    </template>
  </UModal>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import type { InstanceEntry } from '~/types/launcher'

const { isOpen, target, close } = useExportModal()
const activity = useActivityCenter()
const toast = useToast()
const { t } = useI18n()

type Format = 'mrpack' | 'backup'
const format = ref<Format>('mrpack')
const version = ref('1.0.0')
const summary = ref('')
const entries = ref<InstanceEntry[]>([])
const selected = ref<Set<string>>(new Set())
const loading = ref(false)
const exporting = ref(false)
const error = ref<string | null>(null)

const formats = computed<{ value: Format, label: string, desc: string, icon: string, disabled?: boolean }[]>(() => [
  { value: 'mrpack', label: 'Modrinth (.mrpack)', desc: t('export.mrpackDesc'), icon: 'i-lucide-package' },
  { value: 'backup', label: t('export.backup'), desc: t('export.backupDesc'), icon: 'i-lucide-archive' },
])

function fmtSize(n: number) {
  if (n < 1024) return `${n} B`
  if (n < 1024 ** 2) return `${(n / 1024).toFixed(0)} KB`
  if (n < 1024 ** 3) return `${(n / 1024 / 1024).toFixed(1)} MB`
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`
}

function toggle(name: string) {
  const next = new Set(selected.value)
  if (next.has(name)) next.delete(name)
  else next.add(name)
  selected.value = next
}
function selectAll(on: boolean) {
  selected.value = on ? new Set(entries.value.map(e => e.name)) : new Set()
}

async function loadEntries(id: string) {
  loading.value = true
  error.value = null
  try {
    entries.value = await invoke<InstanceEntry[]>('list_instance_entries', { id })
    selected.value = new Set(entries.value.filter(e => e.recommended).map(e => e.name))
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

watch(isOpen, (open) => {
  if (open && target.value) {
    format.value = 'mrpack'
    version.value = '1.0.0'
    summary.value = ''
    error.value = null
    loadEntries(target.value.id)
  }
})

async function doExport() {
  const tgt = target.value
  if (!tgt || !selected.value.size) return
  error.value = null

  const isMrpack = format.value === 'mrpack'
  const ext = isMrpack ? 'mrpack' : 'zip'
  const safe = tgt.name.replace(/[^\w.\- ]+/g, '_').trim() || 'instance'

  try {
    const dest = await save({
      defaultPath: `${safe}.${ext}`,
      filters: [{ name: isMrpack ? 'Modrinth modpack' : 'Mako backup', extensions: [ext] }],
    })
    if (typeof dest !== 'string') return

    exporting.value = true
    const tid = activity.startTask(t('activity.exportingInstance', { name: tgt.name }))
    const include = [...selected.value]
    try {
      if (isMrpack) {
        await invoke('export_mrpack', { id: tgt.id, dest, version: version.value || null, summary: summary.value || null, include })
      } else {
        await invoke('export_instance', { id: tgt.id, dest, include })
      }
      toast.add({ title: t('export.done'), color: 'success' })
      close()
    } finally {
      activity.endTask(tid)
      exporting.value = false
    }
  } catch (e) {
    error.value = String(e)
  }
}
</script>
