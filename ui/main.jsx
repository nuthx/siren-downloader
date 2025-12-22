import "@/globals.css"
import React from "react"
import ReactDOM from "react-dom/client"
import { HashRouter, Routes, Route } from "react-router-dom"
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

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <HashRouter className="container mx-auto bg-red-50">
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
    </HashRouter>
  </React.StrictMode>
)
