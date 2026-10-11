import { convertFileSrc, invoke } from "@tauri-apps/api/core"
import { openUrl } from "@tauri-apps/plugin-opener"
import { Search, X } from "lucide-react"
import { useEffect, useMemo, useRef, useState } from "react"
import { VirtuosoGrid } from "react-virtuoso"
import { Button } from "@/components/button"
import { AlbumCover } from "@/components/cover"
import { Input } from "@/components/input"
import { MusicItem } from "@/components/music"
import { Tab } from "@/components/tab"
import { cn } from "@/utils/cn"
import { getConfig } from "@/utils/config"
import { useAppStore } from "@/utils/store"

export function HomePage() {
  const { hasUpdate, songList, years, status, loading, isDownloading, refreshSongList, downloadSongs } = useAppStore()
  const [selectedYear, setSelectedYear] = useState("all")
  const [selectedFilter, setSelectedFilter] = useState("all")
  const [searchQuery, setSearchQuery] = useState("")
  const [showCover, setShowCover] = useState(true)
  const [selectedSongIds, setSelectedSongIds] = useState([])
  const [nowPlaying, setNowPlaying] = useState(null)
  const [playbackError, setPlaybackError] = useState(null)
  const audioRef = useRef(null)

  // 是否显示封面
  useEffect(() => {
    getConfig().then((config) => setShowCover(config.show_cover))
  }, [])

  const handlePlaySong = async (song) => {
    setPlaybackError(null)
    try {
      const path = await invoke("get_local_song_path", { songId: song.id })
      setNowPlaying({ song, src: convertFileSrc(path) })
    } catch (error) {
      setPlaybackError(String(error))
    }
  }

  useEffect(() => {
    if (!nowPlaying || !audioRef.current) return
    audioRef.current.play().catch((error) => setPlaybackError(String(error)))
  }, [nowPlaying])

  useEffect(() => {
    if (!songList?.length) {
      setSelectedSongIds([])
      return
    }

    const availableSongIds = new Set(songList.map((song) => song.id))
    setSelectedSongIds((current) => current.filter((songId) => availableSongIds.has(songId)))
  }, [songList])

  // 缓存过滤后的歌曲列表
  const filteredSongs = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLocaleLowerCase()
    const albumMap = (songList || []).reduce((map, song) => {
      const { publish, download, album_id = "" } = song
      const matchesSearch =
        !normalizedQuery ||
        [song.title, song.album_title, publish, song.id]
          .filter(Boolean)
          .some((value) => value.toLocaleLowerCase().includes(normalizedQuery))

      if (
        !matchesSearch ||
        (selectedYear !== "all" && !publish?.startsWith(selectedYear)) ||
        (selectedFilter === "pending" && download) ||
        (selectedFilter === "downloaded" && !download)
      ) {
        return map
      }

      const list = map.get(album_id) || []
      list.push(song)
      map.set(album_id, list)
      return map
    }, new Map())

    // 专辑层面倒序显示，歌曲顺序保持正序不变
    return [...albumMap.values()].reverse().flat()
  }, [songList, selectedYear, selectedFilter, searchQuery])

  const selectedSongIdSet = useMemo(() => new Set(selectedSongIds), [selectedSongIds])
  const selectedSongs = useMemo(
    () => (songList || []).filter((song) => selectedSongIdSet.has(song.id)),
    [songList, selectedSongIdSet],
  )

  const toggleSongSelection = (songId) => {
    setSelectedSongIds((current) =>
      current.includes(songId) ? current.filter((id) => id !== songId) : [...current, songId],
    )
  }

  const handleDownloadSelected = async () => {
    const result = await downloadSongs(selectedSongs)
    if (!result?.successSongIds?.length) {
      return
    }

    const successSongIds = new Set(result.successSongIds)
    setSelectedSongIds((current) => current.filter((songId) => !successSongIds.has(songId)))
  }

  if (!songList) {
    return (
      <div className="flex-center flex-col gap-2 h-full">
        <h2 className="text-2xl">暂无数据</h2>
        <span className="text-secondary pb-6">{status?.message}</span>
        <Button onClick={refreshSongList} disabled={loading}>
          获取歌曲列表
        </Button>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex flex-col gap-4 mx-10 px-6 py-5 border border-t-0 backdrop-blur-2xs">
        <div className="flex items-center gap-4">
          <div className="relative flex-1 max-w-120">
            <Search className="absolute left-3 top-3 size-4 text-secondary" />
            <Input
              aria-label="搜索歌曲"
              value={searchQuery}
              onInput={(event) => setSearchQuery(event.target.value)}
              placeholder="搜索歌曲、专辑或年份"
              className="w-full pl-10 pr-10"
            />
            {searchQuery && (
              <button
                type="button"
                aria-label="清除搜索"
                onClick={() => setSearchQuery("")}
                className="absolute right-3 top-3 text-secondary hover:text-primary"
              >
                <X className="size-4" />
              </button>
            )}
          </div>
          <span className="ml-auto text-secondary whitespace-nowrap">显示 {filteredSongs.length} 首</span>
        </div>
        <div className="flex items-center justify-between gap-6">
          <Tab
            items={[{ value: "all", label: "全部年份" }, ...years.map((year) => ({ value: year, label: year }))]}
            value={selectedYear}
            onChange={setSelectedYear}
          />
          <Tab
            items={[
              { value: "all", label: "全部歌曲" },
              { value: "pending", label: "待下载" },
              { value: "downloaded", label: "已下载" },
            ]}
            value={selectedFilter}
            onChange={setSelectedFilter}
          />
        </div>
      </div>

      {filteredSongs.length > 0 ? (
        <VirtuosoGrid
          totalCount={filteredSongs.length}
          listClassName="grid grid-cols-[repeat(auto-fit,minmax(460px,1fr))] gap-2 my-2 pl-10 pr-8"
          style={{ overflowY: "scroll" }}
          itemContent={(index) => (
            <MusicItem
              song={filteredSongs[index]}
              showCover={showCover}
              selected={selectedSongIdSet.has(filteredSongs[index].id)}
              onToggleSelect={toggleSongSelection}
              onPlaySong={handlePlaySong}
            />
          )}
        />
      ) : (
        <div className="flex-center flex-1 text-secondary">没有找到匹配的歌曲</div>
      )}

      {nowPlaying && (
        <div className="flex items-center gap-4 mx-10 px-4 py-3 border-t backdrop-blur-2xs">
          {showCover && (
            <div className="size-10 shrink-0 overflow-hidden">
              <AlbumCover song={nowPlaying.song} className="size-full" />
            </div>
          )}
          <div className="flex flex-col flex-1 min-w-0">
            <span className="text-xs text-secondary">本地播放</span>
            <span className="truncate">{nowPlaying.song.title}</span>
          </div>
          {/* biome-ignore lint/a11y/useMediaCaption: The player is for music, and no synchronized captions are available. */}
          <audio ref={audioRef} controls preload="metadata" src={nowPlaying.src} className="w-96 max-w-[55%] h-10" />
          <button
            type="button"
            aria-label="关闭播放器"
            onClick={() => setNowPlaying(null)}
            className="text-secondary hover:text-primary"
          >
            <X className="size-4" />
          </button>
        </div>
      )}

      <div className="flex-center justify-end gap-4 mx-10 py-6 border-t">
        {playbackError && <span className="flex-1 min-w-0 truncate text-amber-400">{playbackError}</span>}
        {status && (
          <span
            className={cn(
              "flex-1 min-w-0 truncate",
              status?.type === "error" && "text-red-400",
              status?.type === "warning" && "text-amber-400",
            )}
          >
            {status.message}
          </span>
        )}
        {hasUpdate && (
          <Button
            onClick={() => openUrl("https://github.com/nuthx/siren-downloader/releases/latest")}
            className="text-amber-400 border-amber-400/80 hover:text-amber-400/70 hover:border-amber-400/50"
          >
            发现新版本
          </Button>
        )}
        <Button onClick={refreshSongList} disabled={loading}>
          更新歌曲列表
        </Button>
        <Button onClick={handleDownloadSelected} disabled={loading || isDownloading || selectedSongIds.length === 0}>
          下载所选歌曲
        </Button>
        <Button onClick={() => downloadSongs()} disabled={loading || isDownloading}>
          下载所有歌曲
        </Button>
      </div>
    </div>
  )
}
