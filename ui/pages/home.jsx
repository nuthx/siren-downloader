import { openUrl } from "@tauri-apps/plugin-opener"
import { useState, useMemo, useEffect } from "react"
import { VirtuosoGrid } from "react-virtuoso"
import { useAppStore } from "@/utils/store"
import { getConfig } from "@/utils/config"
import { cn } from "@/utils/cn"
import { Button } from "@/components/button"
import { Tab } from "@/components/tab"
import { MusicItem } from "@/components/music"

export function HomePage() {
  const {
    hasUpdate,
    songList,
    years,
    status,
    loading,
    isDownloading,
    refreshSongList,
    downloadSongs
  } = useAppStore()
  const [selectedYear, setSelectedYear] = useState("all")
  const [selectedFilter, setSelectedFilter] = useState("all")
  const [showCover, setShowCover] = useState(true)
  const [selectedSongIds, setSelectedSongIds] = useState([])

  // 是否显示封面
  useEffect(() => {
    getConfig().then((config) => setShowCover(config.show_cover))
  }, [])

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
    const albumMap = (songList || []).reduce((map, song) => {
      const { publish, download, album_id = "" } = song

      if (
        (selectedYear !== "all" && !publish?.startsWith(selectedYear))
        || (selectedFilter === "pending" && download)
        || (selectedFilter === "downloaded" && !download)
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
  }, [songList, selectedYear, selectedFilter])

  const selectedSongIdSet = useMemo(() => new Set(selectedSongIds), [selectedSongIds])
  const selectedSongs = useMemo(
    () => (songList || []).filter((song) => selectedSongIdSet.has(song.id)),
    [songList, selectedSongIdSet]
  )

  const toggleSongSelection = (songId) => {
    setSelectedSongIds((current) => (
      current.includes(songId)
        ? current.filter((id) => id !== songId)
        : [...current, songId]
    ))
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
        <Button onClick={refreshSongList} disabled={loading}>获取歌曲列表</Button>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex-center justify-between mx-10 px-6 py-5 border border-t-0 backdrop-blur-2xs">
        <Tab
          items={[
            { value: "all", label: "全部年份" },
            ...years.map((year) => ({ value: year, label: year }))
          ]}
          value={selectedYear}
          onChange={setSelectedYear}
        />
        <Tab
          items={[
            { value: "all", label: "全部歌曲" },
            { value: "pending", label: "待下载" },
            { value: "downloaded", label: "已下载" }
          ]}
          value={selectedFilter}
          onChange={setSelectedFilter}
        />
      </div>

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
          />
        )}
      />

      <div className="flex-center justify-end gap-4 mx-10 py-6 border-t">
        {status && <span className={cn("flex-1 min-w-0 truncate", status?.type === "error" && "text-red-400", status?.type === "warning" && "text-amber-400")}>{status.message}</span>}
        {hasUpdate && (
          <Button
            onClick={() => openUrl("https://github.com/nuthx/siren-downloader/releases/latest")}
            className="text-amber-400 border-amber-400/80 hover:text-amber-400/70 hover:border-amber-400/50"
          >
            发现新版本
          </Button>
        )}
        <Button onClick={refreshSongList} disabled={loading}>更新歌曲列表</Button>
        <Button onClick={handleDownloadSelected} disabled={loading || isDownloading || selectedSongIds.length === 0}>下载所选歌曲</Button>
        <Button onClick={() => downloadSongs()} disabled={loading || isDownloading}>下载所有歌曲</Button>
      </div>
    </div>
  )
}
