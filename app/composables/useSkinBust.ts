// Renders a "bust" (head + torso + arms, cut at the waist, slight 3/4 angle) from
// a skin texture, like crafty.gg's bust renders — but locally, so it works for
// any saved/default skin file (not just a named Mojang profile).
//
// One hidden, paused SkinViewer is reused for every render (one WebGL context);
// requests are serialised since a single viewer can't render two skins at once.

// eslint-disable-next-line @typescript-eslint/no-explicit-any
let viewerPromise: Promise<any> | null = null
let queue: Promise<unknown> = Promise.resolve()

async function getBustViewer() {
  if (!viewerPromise) {
    viewerPromise = (async () => {
      const { SkinViewer } = await import('skinview3d')
      const canvas = document.createElement('canvas')
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const v: any = new SkinViewer({ canvas, width: 160, height: 160, renderPaused: true })
      // Bust = no legs; slight turn for a 3/4 view.
      v.playerObject.skin.leftLeg.visible = false
      v.playerObject.skin.rightLeg.visible = false
      v.playerObject.cape.visible = false
      v.playerObject.elytra.visible = false
      v.playerObject.rotation.y = -0.4
      return v
    })()
  }
  return viewerPromise
}

export function useSkinBust() {
  /** Renders `source` (data/remote URL) to a PNG data URL of the bust. */
  function render(source: string, model: 'classic' | 'slim'): Promise<string> {
    const task = queue.then(async () => {
      const v = await getBustViewer()
      await v.loadSkin(source, { model: model === 'slim' ? 'slim' : 'default' })
      // Frame the upper body: look at chest height, far enough to keep the head
      // in frame and cut roughly at the waist.
      const cam = v.camera
      cam.fov = 45
      const centerY = 6
      cam.position.set(0, centerY, 29)
      cam.lookAt(0, centerY, 0)
      cam.updateProjectionMatrix()
      v.render()
      return v.canvas.toDataURL('image/png') as string
    })
    queue = task.catch(() => {})
    return task
  }

  return { render }
}
