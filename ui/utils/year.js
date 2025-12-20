export function extractYears(songs) {
  const years = new Set()

  for (const s of songs) {
    if (s.publish) {
      years.add(s.publish.slice(0, 4))
    }
  }

  return [...years].sort((a, b) => b.localeCompare(a))
}
