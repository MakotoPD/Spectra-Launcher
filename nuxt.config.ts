// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  ssr: false, 
  app: {
    head: {
      // CSP musi zezwalać na zasoby Tauri
    }
  },

  vite: {
    // Tauri potrzebuje stałego portu w dev
    server: {
      strictPort: true,
       watch: {
        ignored: ['**/src-tauri/**']
      },
      hmr: {
        protocol: 'ws',
        host: '0.0.0.0',
        port: 3001,
      },
    },
    envPrefix: ['VITE_', 'TAURI_'],
    // Pre-bundle the drag-and-drop lib so it works without a manual dev restart.
    optimizeDeps: {
      include: ['vue-draggable-plus'],
    },
  },

  nitro: {
    static: true,
    ignore: ['src-tauri/**']
  },

  css: ['~/assets/css/main.css'],
  
  modules: [
    '@pinia/nuxt',
    '@nuxt/scripts',
    '@nuxt/ui',
    '@nuxtjs/i18n',
  ],

  // Desktop app: keep URLs clean (no /en/ prefix). Language is switched in-app
  // and remembered in a cookie; `no_prefix` also stops i18n from rewriting page
  // routes (the default `prefix_except_default` strategy breaks NuxtLink nav).
  //
  // Adding a language = drop a JSON file in i18n/locales/ and add one entry to
  // `locales` below. Nothing else to wire up.
  i18n: {
    strategy: 'no_prefix',
    defaultLocale: 'en',
    lazy: true,
    locales: [
      { code: 'en', name: 'English', file: 'en.json' },
      { code: 'pl', name: 'Polski', file: 'pl.json' },
      { code: 'de', name: 'Deutsch', file: 'de.json' },
      { code: 'es', name: 'Español', file: 'es.json' },
      { code: 'fr', name: 'Français', file: 'fr.json' },
    ],
    // Auto-detect the OS/browser language on first run, then remember the
    // user's choice across launches via a cookie.
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: 'spectra_locale',
      fallbackLocale: 'en',
      alwaysRedirect: false,
    },
  },
})