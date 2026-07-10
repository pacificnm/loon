export interface PlayerRoute {
  slug: string
  title: string
  streamUrl?: string
}

/** Parses `player.html?slug=…&title=…&streamUrl=…`. */
export function parsePlayerQuery(): PlayerRoute | null {
  const params = new URLSearchParams(window.location.search)
  const slug = params.get('slug')?.trim()
  if (!slug) {
    return null
  }
  const title = params.get('title')?.trim() || slug
  const streamUrl = params.get('streamUrl')?.trim() || undefined
  return { slug, title, streamUrl }
}

export interface PlayerLoadPayload {
  slug: string
  title: string
  streamUrl: string
}
