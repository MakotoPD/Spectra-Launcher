<template>
  <button
    type="button"
    class="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition"
    :class="active ? 'bg-primary-500/10' : 'hover:bg-white/5'"
    @click="onClick"
  >
    <InstanceIcon :instance="instance" class="size-7 rounded-md text-[12px]" />
    <div class="min-w-0 flex-1">
      <div class="truncate text-[13px] font-medium">{{ instance.name }}</div>
      <div class="font-mono text-[10px] text-neutral-500">{{ instance.mc_version }}</div>
    </div>
  </button>
</template>

<script setup lang="ts">
import type { Instance } from '~/types/launcher'

const props = defineProps<{ instance: Instance }>()

const router = useRouter()
const route = useRoute()

const active = computed(() => route.params.id === props.instance.id)

const onClick = () => {
  router.push(`/instance/${props.instance.id}`)
}
</script>
