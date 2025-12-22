import { invoke, convertFileSrc } from "@tauri-apps/api/core"
import { useState, useEffect } from "react"
import { cn } from "@/utils/cn"
import { ImageOff, LoaderCircle } from "lucide-react"

const coverCache = new Map()

export function AlbumCover({ song, className }) {
  const [src, setSrc] = useState(() => coverCache.get(song?.album_id) || null)
  const [error, setError] = useState(false)

  useEffect(() => {
    if (!song?.album_id || !song?.cover_ncm || src || error) return

    ;(async () => {
      try {
        const coverPath = await invoke("get_cover", {
          albumId: song.album_id,
          coverUrl: song.cover_ncm
        })
        const result = convertFileSrc(coverPath)
        coverCache.set(song.album_id, result)
        setSrc(result)
      } catch {
        setError(true)
      }
    })()
  }, [song?.album_id, song?.cover_ncm, src, error])

  if (error) {
    return (
      <div className={`flex-center bg-muted/30 ${className}`}>
        <ImageOff strokeWidth={1.5} className="size-5 text-muted" />
      </div>
    )
  }

  if (!src) {
    return (
      <div className={`flex-center bg-muted/30 ${className}`}>
        <LoaderCircle strokeWidth={1.5} className="size-5 text-muted animate-spin" />
      </div>
    )
  }

  return <img src={src} alt={song.title} className={cn("object-cover", className)} draggable="false" />
}
