// Applies the persisted theme (mode + accent) as early as possible on the
// client so the whole app starts in the right colors.
export default defineNuxtPlugin(() => {
  const theme = useThemeStore()
  theme.apply()
})
