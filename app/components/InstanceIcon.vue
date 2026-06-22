<template>
  <div
    class="flex flex-none items-center justify-center overflow-hidden font-bold text-white"
    :style="bgStyle"
  >
    <img v-if="src" :src="src" :alt="instance.name" draggable="false" class="size-full object-cover" />
    <template v-else>{{ initial }}</template>
  </div>
</template>

<script setup lang="ts">
import type { Instance } from '~/types/launcher'

// Shows the instance's icon.png (via the asset protocol) when it has one,
// otherwise a colored initial. Size/rounding/font come from the parent's class.
const props = defineProps<{ instance: Pick<Instance, 'id' | 'name' | 'icon'> }>()

const src = ref<string | null>(null)

watch(
  () => [props.instance.id, props.instance.icon] as const,
  async () => {
    src.value = await resolveInstanceIcon(props.instance.id, !!props.instance.icon)
  },
  { immediate: true },
)

const initial = computed(() => instanceInitial(props.instance))
const bgStyle = computed(() => (src.value ? {} : { background: instanceIconBg(props.instance) }))
</script>
