import { Button } from "@/components/button"
import { AlbumCover } from "@/components/cover"
import { ArrowDownToLine, CheckLine } from "lucide-react"

export function MusicItem({ song, loading, isDownloading, onDownload, showCover }) {
  const isInstrumental = song.instrumental

  return (
    <div className="relative flex gap-4 items-center h-fit p-4 pr-6 border backdrop-blur-2xs">
      {showCover && <AlbumCover song={song} className="size-20" />}

      {song.download && <div className="absolute top-0 right-0 border-t-10 border-l-10 border-t-lime-500 border-l-transparent" />}

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

      <Button onClick={() => onDownload(song)} disabled={loading || isDownloading || song.download} className="w-11">
        {song.download ? <CheckLine className="size-3.5 shrink-0" /> : <ArrowDownToLine className="size-3.5 shrink-0" /> }
      </Button>
    </div>
  )
}
