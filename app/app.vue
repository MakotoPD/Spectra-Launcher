<template>
  <UApp class="overflow-hidden">
    <!-- Titlebar -->
    <div data-tauri-drag-region class="z-10 flex w-full justify-between items-center h-10 px-2 text-gray-100 select-none">
      <div class="flex items-center gap-2 pl-2">
        <img src="/logo-transparent.png" alt="Spectra Launcher Icon" class="h-5 object-contain" />
        <span>Spectra Launcher</span>
      </div>
      <div class="flex items-center gap-4">
        <TitlebarActivity />
        <WindowControls />
      </div>
    </div>


    <NuxtLoadingIndicator color="aqua" errorColor="red" />
    <div :class="['relative w-screen h-[calc(100vh-2.5rem)] overflow-hidden text-[#eef1f5]', theme.bgClass]">


      <!-- Background texture + animated glows (from the Spectra design) -->
      <div
        class="pointer-events-none absolute inset-0"
        style="background-image:radial-gradient(rgba(255,255,255,0.035) 1px,transparent 1px);background-size:26px 26px;"
      />

      <div class="relative z-[1] h-full">
        <NuxtLayout>
          <NuxtPage />
        </NuxtLayout>
      </div>
    </div>

    <LiveLogsModal />
    <CrashReportModal />
  </UApp>
</template>

<script setup lang="ts">
// Theme is applied in plugins/theme.client.ts; here we just expose the reactive
// background class to the shell.
const theme = useThemeStore()

// Attach the global launch/install event hub once, so the titlebar activity
// indicator works regardless of which page is open.
const activity = useActivityCenter()
const instances = useInstancesStore()
const updater = useAutoUpdate()
const telemetry = useTelemetry()
onMounted(() => {
  activity.attach()
  // Needed so the indicator can resolve instance names.
  instances.ensureLoaded()
  // Quietly look for a new release; surfaces an "Update" button in Settings.
  updater.checkForUpdates(true)
  // Anonymous usage stats (no-op unless opted in via Settings → Privacy).
  telemetry.init()
})
onBeforeUnmount(() => activity.detach())
</script>
