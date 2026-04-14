import packageJson from "#/package.json"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { create } from "zustand"
import { extractYears } from "@/utils/year"

export const useAppStore = create((set) => ({
  hasUpdate: false, // 是否有更新
  songList: null, // 歌曲列表
  years: [], // 年份列表
  status: null, // 左下角的状态显示
  loading: false, // 是否正在手动刷新歌曲列表
  isDownloading: false, // 是否有歌曲正在下载

  // 初始化应用数据
  initAppData: async () => {
    // 检查更新（独立执行，不阻塞其他初始化）
    invoke("load_latest_version").then((latestVersion) => {
      if (latestVersion && latestVersion.trim() !== packageJson.version.trim()) {
        set({ hasUpdate: true })
      }
    }).catch((error) => {
      console.error("检查更新失败:", error)
    })

    // 同步加载本地和远程歌曲数据
    const [localSongs, remoteSongs] = await Promise.all([
      invoke("load_local_music").catch(() => null),
      invoke("load_remote_music").catch(() => null)
    ])

    // 先处理本地歌曲数据，如果没有则提前返回
    if (!localSongs || localSongs.length === 0) {
      set({ status: { type: "default", message: "请先点击按钮获取歌曲列表" } })
      return
    }
    set({
      songList: localSongs,
      years: extractYears(localSongs)
    })

    if (!remoteSongs) {
      set({ status: { type: "error", message: `已加载本地 ${localSongs.length} 首歌曲，但无法获取官网的歌曲信息` } })
    } else if (remoteSongs.length !== localSongs.length) {
      set({ status: { type: "warning", message: `已加载本地 ${localSongs.length} 首歌曲，但官网有 ${remoteSongs.length} 首歌曲。请先更新歌曲列表` } })
    } else {
      set({ status: { type: "default", message: `已加载本地 ${localSongs.length} 首歌曲` } })
    }
  },

  // 刷新歌曲列表
  refreshSongList: async () => {
    set({
      loading: true,
      status: { type: "default", message: "正在更新歌曲列表" }
    })
    try {
      const result = await invoke("refresh_music_list")
      set({
        songList: result,
        years: extractYears(result),
        status: { type: "default", message: `更新完成，已保存 ${result.length} 首歌曲信息` }
      })
    } catch (error) {
      set({ status: { type: "error", message: `更新失败: ${error}` } })
    } finally {
      set({ loading: false })
    }
  },

  // 在前端更新歌曲下载状态（仅前端）
  updateSongStatus: (songId, downloaded) => {
    set((state) => ({
      songList: state.songList?.map((s) => s.id === songId ? { ...s, download: downloaded } : s) ?? state.songList
    }))
  },

  // 在前端批量更新歌曲下载状态（仅前端）
  setAllSongStatuses: (downloaded) => {
    set((state) => ({
      songList: state.songList?.map((song) => ({ ...song, download: downloaded })) ?? state.songList
    }))
  },

  // 下载单首歌曲
  downloadSong: async (song) => {
    const { updateSongStatus } = useAppStore.getState()
    set({ isDownloading: true, status: { type: "default", message: `下载中: ${song.title}` } })
    try {
      await invoke("download_music", { songId: song.id })
      updateSongStatus(song.id, true)
      set({ status: { type: "default", message: `下载完成: ${song.title}` } })
    } catch (error) {
      set({ status: { type: "error", message: `下载失败: ${error}` } })
    } finally {
      set({ isDownloading: false })
    }
  },

  // 下载所有歌曲
  downloadAllSongs: async () => {
    const { updateSongStatus } = useAppStore.getState()
    // 监听下载进度
    const unlisten = await listen("download-progress", (event) => {
      const { current, total, song_id, song_title, success } = event.payload
      set({ status: { type: "default", message: `下载中 (${current}/${total}): ${song_title}` } })
      if (success) {
        updateSongStatus(song_id, true)
      }
    })

    set({ isDownloading: true, loading: true, status: { type: "default", message: "正在批量下载..." } })
    try {
      const [successCount, failCount] = await invoke("download_all_music")
      if (successCount === 0 && failCount === 0) {
        set({ status: { type: "default", message: "没有需要下载的歌曲" } })
      } else if (failCount === 0) {
        set({ status: { type: "default", message: `下载完成，共 ${successCount} 首歌曲` } })
      } else {
        set({ status: { type: "warning", message: `下载完成: ${successCount} 首成功，${failCount} 首失败` } })
      }
    } catch (error) {
      set({ status: { type: "error", message: `下载失败: ${error}` } })
    } finally {
      unlisten()
      set({ isDownloading: false, loading: false })
    }
  }
}))
