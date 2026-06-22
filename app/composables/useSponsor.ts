/** The sponsored modpack featured on the home page. */
export interface Sponsor {
  /** Modrinth project id. */
  projectId: string
  slug: string
  title: string
  description: string
  iconUrl: string
  url: string
}

export const SPONSOR: Sponsor = {
  projectId: 'yO4uKECz',
  slug: 'zebatkowo',
  title: 'Zębatkowo',
  description: 'The modpack offers an unforgettable story that you can even experience with your friends!',
  iconUrl: 'https://cdn.modrinth.com/data/yO4uKECz/4dd75fd866111020c81f5a393aa1dc34d6d0fe8a_96.webp',
  url: 'https://modrinth.com/modpack/zebatkowo',
}

const KEY = 'mako-sponsor-hidden'

/**
 * Visibility state for the sponsor section on the home page. It's toggled only
 * from settings (no in-section close button). A UI preference, so it lives in
 * localStorage rather than launcher.json.
 */
export const useSponsor = () => {
  const dismissed = useState('sponsor-dismissed', () => {
    if (!import.meta.client) return false
    return localStorage.getItem(KEY) === '1'
  })

  function dismiss() {
    dismissed.value = true
    if (import.meta.client) localStorage.setItem(KEY, '1')
  }

  function restore() {
    dismissed.value = false
    if (import.meta.client) localStorage.removeItem(KEY)
  }

  return { sponsor: SPONSOR, dismissed, dismiss, restore }
}
