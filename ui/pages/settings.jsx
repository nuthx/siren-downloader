import { useState, useEffect } from "react"
import { appDataDir } from "@tauri-apps/api/path"
import { invoke } from "@tauri-apps/api/core"
import { open } from "@tauri-apps/plugin-dialog"
import { openPath } from "@tauri-apps/plugin-opener"
import { DEFAULT_CONFIG, getConfig, saveConfig } from "@/utils/config"
import { cn } from "@/utils/cn"
import { Button } from "@/components/button"
import { Input } from "@/components/input"
import { Select } from "@/components/select"
import { SettingsItem } from "@/components/settings"

export function SettingsPage() {
  const [config, setConfig] = useState(null)
  const [status, setStatus] = useState(null)
  const [isWindows, setIsWindows] = useState(false)
  const [isDownloading, setIsDownloading] = useState(false)

  useEffect(() => {
    // 加载配置
    getConfig().then(setConfig).catch((error) => {
      setStatus({ type: "error", message: `配置加载失败: ${error}` })
    })

    // 检测操作系统
    invoke("get_platform").then((platform) => {
      setIsWindows(platform === "windows")
    })
  }, [])

  // 处理配置项改变
  const handleChange = (key, value) => {
    setConfig({ ...config, [key]: value })
  }

  // 获取 FFmpeg 版本
  const handleCheckVersion = async () => {
    try {
      const version = await invoke("get_ffmpeg_version")
      setStatus({ type: "default", message: `当前版本: ${version}` })
    } catch (error) {
      setStatus({ type: "error", message: error })
    }
  }

  // 下载 FFmpeg
  const handleDownloadFFmpeg = async () => {
    setIsDownloading(true)
    setStatus({ type: "default", message: "正在下载 FFmpeg，请稍候..." })
    try {
      await invoke("download_ffmpeg")
      const version = await invoke("get_ffmpeg_version")
      setStatus({ type: "default", message: `下载成功: ${version}` })
    } catch (error) {
      setStatus({ type: "error", message: `下载失败: ${error}` })
    } finally {
      setIsDownloading(false)
    }
  }

  // 删除 FFmpeg
  const handleDeleteFFmpeg = async () => {
    try {
      await invoke("delete_ffmpeg")
      setStatus({ type: "default", message: "FFmpeg 已删除" })
    } catch (error) {
      setStatus({ type: "error", message: `删除失败: ${error}` })
    }
  }

  // 选择下载文件夹
  const handleSelectFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false })
      if (selected) {
        handleChange("download_path", selected)
        setStatus({ type: "warning", message: "下载路径已配置，点击保存后才可生效" })
      }
    } catch (error) {
      setStatus({ type: "error", message: `选择失败: ${error}` })
    }
  }

  // 打开配置文件夹
  const handleOpenConfig = async () => {
    try {
      const configDir = await appDataDir()
      await openPath(configDir)
    } catch (error) {
      setStatus({ type: "error", message: `打开失败: ${error}` })
    }
  }

  // 重置配置
  const handleReset = () => {
    setConfig({ ...DEFAULT_CONFIG })
    setStatus({ type: "warning", message: "已重置为默认配置，需要点击保存后才可生效" })
  }

  // 保存配置
  const handleSubmit = async (e) => {
    e.preventDefault()
    try {
      await saveConfig(config)
      setStatus({ type: "default", message: "配置已保存" })
      setTimeout(() => setStatus(null), 5000)
    } catch (error) {
      setStatus({ type: "error", message: `保存失败: ${error}` })
    }
  }

  if (!config) {
    return <div className="flex-center min-h-full">{status?.message}</div>
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col h-full overflow-hidden">
      <div className="flex-1 p-8 pl-10 overflow-y-scroll">
        <div className="flex flex-col gap-8 max-w-150 mx-auto">
          <SettingsItem title="FFmpeg 管理" desc="转换音频格式并缩放封面图片" className="gap-2">
            {isWindows && (
              <>
                <Button type="button" onClick={handleDownloadFFmpeg} disabled={isDownloading} className="border-border hover:border-border">
                  下载 FFmpeg
                </Button>
                <Button type="button" onClick={handleDeleteFFmpeg} disabled={isDownloading} className="border-border hover:border-border">
                  删除 FFmpeg
                </Button>
              </>
            )}
            <Button type="button" onClick={handleCheckVersion} disabled={isDownloading} className="border-border hover:border-border">
              查看版本
            </Button>
          </SettingsItem>

          <SettingsItem title="下载路径" desc="指定歌曲文件的下载路径">
            <Input value={config.download_path} onInput={(e) => handleChange("download_path", e.target.value)} placeholder="请选择下载路径" />
            <Button type="button" onClick={handleSelectFolder} className="border-l-0 border-border hover:border-border">选择文件夹</Button>
          </SettingsItem>

          <SettingsItem title="下载伴奏" desc="是否下载标题包含 Instrumental 的歌曲">
            <Select
              options={[
                { value: false, label: "不下载伴奏" },
                { value: true, label: "下载伴奏" }
              ]}
              value={config.download_instrumental}
              onChange={(value) => handleChange("download_instrumental", value)}
            />
          </SettingsItem>

          <SettingsItem title="下载歌词" desc="是否从官网同步下载歌词文件">
            <Select
              options={[
                { value: false, label: "不下载歌词" },
                { value: true, label: "下载歌词" }
              ]}
              value={config.download_lyrics}
              onChange={(value) => handleChange("download_lyrics", value)}
            />
          </SettingsItem>

          <SettingsItem title="显示封面" desc="是否在首页显示专辑封面，开启后可能会导致程序卡顿">
            <Select
              options={[
                { value: true, label: "显示封面" },
                { value: false, label: "隐藏封面" }
              ]}
              value={config.show_cover}
              onChange={(value) => handleChange("show_cover", value)}
            />
          </SettingsItem>

          {import.meta.env.DEV && (
            <SettingsItem title="配置文件">
              <Button type="button" onClick={handleOpenConfig} className="border-border hover:border-border">打开配置文件夹</Button>
            </SettingsItem>
          )}
        </div>
      </div>

      <div className="flex-center justify-end gap-4 mx-10 py-6 border-t">
        {status && <span className={cn("flex-1", status?.type === "error" && "text-red-400", status?.type === "warning" && "text-amber-400")}>{status.message}</span>}
        <Button type="button" onClick={handleReset}>重置</Button>
        <Button type="submit">保存设置</Button>
      </div>
    </form>
  )
}
