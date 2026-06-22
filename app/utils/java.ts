import type { JavaInstallation } from '~/composables/useJava'

/**
 * The Java major version a Minecraft version needs (matches what the engine
 * provisions automatically):
 *   - < 1.17        → Java 8
 *   - 1.17 – 1.20.4 → Java 17
 *   - ≥ 1.20.5      → Java 21
 */
export function requiredJavaMajor(mcVersion: string): number {
  const m = mcVersion.match(/^1\.(\d+)(?:\.(\d+))?/)
  if (!m) return 21 // snapshots / unknown → newest
  const minor = Number(m[1])
  const patch = Number(m[2] ?? 0)
  if (minor < 17) return 8
  if (minor < 20) return 17
  if (minor === 20) return patch >= 5 ? 21 : 17
  return 21
}

/** Finds the player's detected Java that satisfies `major` (21 allows newer). */
export function matchJava(installations: JavaInstallation[], major: number): JavaInstallation | undefined {
  return installations.find(
    j => j.is_valid && (j.major === major || (major === 21 && (j.major ?? 0) >= 21)),
  )
}
