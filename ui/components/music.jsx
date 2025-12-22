import { listen } from "@tauri-apps/api/event"
import { useState, useEffect } from "react"
import { AlbumCover } from "@/components/cover"
import { DropdownMenu } from "@/components/menu"

export function MusicItem({ song, loading, isDownloading, onDownload, showCover }) {
  const isInstrumental = song.instrumental
  const [downloadProgress, setDownloadProgress] = useState(null)

  useEffect(() => {
    const unlisten = listen("download-single-progress", (event) => {
      const { song_id, progress } = event.payload
      if (song_id === song.id) {
        if (progress >= 100) {
          setDownloadProgress(100)
          setTimeout(() => {
            setDownloadProgress(null)
          }, 200) // 添加延迟，避免进度条没到头就消失
        } else {
          setDownloadProgress(progress)
        }
      }
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [song.id])

  return (
    <div className="relative flex gap-4 items-center h-fit p-4 border backdrop-blur-2xs">
      {showCover && <AlbumCover song={song} className="size-20" />}

      {song.download && <div className="absolute top-0 right-0 border-t-10 border-l-10 border-t-lime-400 border-l-transparent" />}

      {downloadProgress !== null && (
        <div
          className="absolute left-0 bottom-0 h-0.75 bg-lime-400 transition"
          style={{ width: `${downloadProgress}%` }}
        />
      )}

      <div className="flex-1 flex flex-col gap-1 min-w-0">
        <h5 className="text-base font-semibold">{song.title}</h5>
        <div className="flex items-center gap-3 text-secondary">
          {isInstrumental && <span className="text-amber-400">伴奏</span>}
          {isInstrumental && <div className="w-px h-3 bg-secondary rotate-16" />}
          <span>{song.publish}</span>
          <div className="w-px h-3 bg-secondary rotate-16" />
          <span className="flex-1 truncate whitespace-nowrap">{song.album_title}</span>
        </div>
      </div>

      <DropdownMenu
        items={[
          {
            label: song.download ? "重新下载歌曲" : "下载歌曲",
            disabled: loading || isDownloading,
            onClick: () => onDownload(song)
          }
        ]}
      />
    </div>
  )
}
