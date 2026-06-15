import "@/globals.css"
import { StrictMode, useEffect } from "react"
import { createRoot } from "react-dom/client"
import { HashRouter, Routes, Route } from "react-router-dom"
import { listen } from "@tauri-apps/api/event"
import { initConfig } from "@/utils/config"
import { useAppStore } from "@/utils/store"
import { AppWindow, MainContent } from "@/components/window"
import { Nav, NavButton } from "@/components/nav"
import { HomePage } from "@/pages/home"
import { SettingsPage } from "@/pages/settings"
import { AboutPage } from "@/pages/about"

// 初始化配置和应用数据
initConfig().catch(console.error)
useAppStore.getState().initAppData()

function App() {
  useEffect(() => {
    let unlistenNavigate
    let unlistenMenuAction

    // 监听菜单跳转
    listen("navigate", (event) => {
      window.location.hash = event.payload
    }).then((unlisten) => {
      unlistenNavigate = unlisten
    })

    // 监听菜单功能
    listen("menu-action", (event) => {
      const { refreshSongList, downloadSongs } = useAppStore.getState()
      if (event.payload === "refresh") refreshSongList()
      else if (event.payload === "download_all") downloadSongs()
    }).then((unlisten) => {
      unlistenMenuAction = unlisten
    })

    return () => {
      if (unlistenNavigate) unlistenNavigate()
      if (unlistenMenuAction) unlistenMenuAction()
    }
  }, [])

  return (
    <AppWindow>
      <Nav>
        <NavButton path="/" title="首页" />
        <NavButton path="/settings" title="设置" />
        <NavButton path="/about" title="关于" />
      </Nav>
      <MainContent>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/about" element={<AboutPage />} />
        </Routes>
      </MainContent>
    </AppWindow>
  )
}

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <HashRouter>
      <App />
    </HashRouter>
  </StrictMode>
)
