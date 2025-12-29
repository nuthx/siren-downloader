import { useState, useEffect } from "react"
import { appDataDir } from "@tauri-apps/api/path"
import { invoke } from "@tauri-apps/api/core"
import { platform } from "@tauri-apps/plugin-os"
import { open } from "@tauri-apps/plugin-dialog"
import { openPath } from "@tauri-apps/plugin-opener"
import { getConfig, saveConfig, resetConfig } from "@/utils/config"
import { cn } from "@/utils/cn"
import { Button } from "@/components/button"
import { Input } from "@/components/input"
import { Select } from "@/components/select"
import { SettingsItem } from "@/components/settings"

export function SettingsPage() {
  const [config, setConfig] = useState(null)
  const [status, setStatus] = useState(null)
  const [isDownloading, setIsDownloading] = useState(false)

  // 加载配置
  useEffect(() => {
    getConfig().then(setConfig).catch((error) => {
      setStatus({ type: "error", message: `配置加载失败: ${error}` })
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
  const handleReset = async () => {
    try {
      await resetConfig()
      setConfig(await getConfig())
      setStatus({ type: "warning", message: "已重置为默认配置，需要点击保存后才可生效" })
    } catch (error) {
      setStatus({ type: "error", message: `重置失败: ${error}` })
    }
  }

  // 保存配置
  const handleSubmit = async (e) => {
    e.preventDefault()
    try {
      await saveConfig(config)
      setStatus({ type: "default", message: "配置已保存" })
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
          <SettingsItem title="FFmpeg 管理" desc="用于转换音频格式和压缩封面图片，使用前请确保已正确安装" className="gap-2">
            {platform() === "windows" && (
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

          <SettingsItem title="存储位置" desc="歌曲将保存至该目录下的「明日方舟」文件夹中">
            <Input value={config.download_path} onInput={(e) => handleChange("download_path", e.target.value)} placeholder="请选择下载路径" />
            <Button type="button" onClick={handleSelectFolder} className="border-l-0 border-border hover:border-border">选择文件夹</Button>
          </SettingsItem>

          <SettingsItem title="下载伴奏" desc="是否同时下载标题含 Instrumental 的伴奏版本">
            <Select
              options={[
                { value: false, label: "不下载伴奏" },
                { value: true, label: "下载伴奏" }
              ]}
              value={config.download_instrumental}
              onChange={(value) => handleChange("download_instrumental", value)}
            />
          </SettingsItem>

          <SettingsItem title="下载歌词" desc="是否从官网同步获取并下载 LRC 歌词">
            <Select
              options={[
                { value: false, label: "不下载歌词" },
                { value: true, label: "下载歌词" }
              ]}
              value={config.download_lyrics}
              onChange={(value) => handleChange("download_lyrics", value)}
            />
          </SettingsItem>

          <SettingsItem title="自定义专辑" desc="是否在专辑文件夹名称前添加日期前缀">
            <Select
              options={[
                { value: "none", label: "仅专辑名：焰烬曙明" },
                { value: "year", label: "年份前缀：[2025] 焰烬曙明" },
                { value: "lite", label: "简单日期前缀：[250905] 焰烬曙明" },
                { value: "full", label: "完整日期前缀：[2025-09-05] 焰烬曙明" }
              ]}
              value={config.custom_album}
              onChange={(value) => handleChange("custom_album", value)}
            />
          </SettingsItem>

          <SettingsItem title="显示专辑封面" desc="在首页列表中显示封面图片，此选项开启时可能会影响性能">
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
