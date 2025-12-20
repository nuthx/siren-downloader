import { useState, useEffect } from "react"
import { appDataDir } from "@tauri-apps/api/path"
import { open } from "@tauri-apps/plugin-dialog"
import { openPath } from "@tauri-apps/plugin-opener"
import { DEFAULT_CONFIG, getConfig, saveConfig } from "@/utils/config"
import { Button } from "@/components/button"
import { Input } from "@/components/input"
import { Select } from "@/components/select"
import { SettingsItem } from "@/components/settings"

export function SettingsPage() {
  const [config, setConfig] = useState(null)
  const [status, setStatus] = useState("")

  // 加载配置
  useEffect(() => {
    getConfig().then(setConfig).catch((error) => {
      setStatus(`配置加载失败: ${error}`)
    })
  }, [])

  // 处理配置项改变
  const handleChange = (key, value) => {
    setConfig({ ...config, [key]: value })
  }

  // 选择下载文件夹
  const handleSelectFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false })
      if (selected) {
        handleChange("download_path", selected)
      }
    } catch (error) {
      setStatus(`选择失败: ${error}`)
    }
  }

  // 打开配置文件夹
  const handleOpenConfig = async () => {
    try {
      const configDir = await appDataDir()
      await openPath(configDir)
    } catch (error) {
      setStatus(`打开失败: ${error}`)
    }
  }

  // 重置配置
  const handleReset = () => {
    setConfig({ ...DEFAULT_CONFIG })
    setStatus("已重置为默认配置，点击保存后才可生效")
  }

  // 保存配置
  const handleSubmit = async (e) => {
    e.preventDefault()
    try {
      await saveConfig(config)
      setStatus("配置已保存")
      setTimeout(() => setStatus(""), 5000)
    } catch (error) {
      setStatus(`保存失败: ${error}`)
    }
  }

  if (!config) {
    return <div className="flex-center min-h-full">{status}</div>
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col h-full overflow-hidden">
      <div className="flex-1 p-8 pl-10 overflow-y-scroll">
        <div className="flex flex-col gap-8 max-w-150 mx-auto">
          <SettingsItem title="网易云 API" desc="用于获取歌曲文件的高清封面与元数据">
            <Input value={config.ncm_api} onInput={(e) => handleChange("ncm_api", e.target.value)} />
          </SettingsItem>

          <SettingsItem title="下载路径" desc="指定歌曲文件的下载路径">
            <Input value={config.download_path} onInput={(e) => handleChange("download_path", e.target.value)} placeholder="请选择下载路径" />
            <Button type="button" onClick={handleSelectFolder} className="border-l-0 border-border hover:border-border">选择文件夹</Button>
          </SettingsItem>

          <SettingsItem title="下载伴奏曲目" desc="是否下载标题包含 Instrumental 的曲目">
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
        {status && <span className="flex-1">{status}</span>}
        <Button type="button" onClick={handleReset}>重置</Button>
        <Button type="submit">保存设置</Button>
      </div>
    </form>
  )
}
