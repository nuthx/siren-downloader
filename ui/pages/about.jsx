import packageJson from "#/package.json"
import { openUrl } from "@tauri-apps/plugin-opener"

export function AboutPage() {
  return (
    <div className="flex-center flex-col gap-16 h-full">
      <div className="flex-center flex-col gap-2">
        <img src="/logo/siren.svg" alt="logo" className="w-54" />
        <h1 className="text-xl font-bold">明日方舟塞壬唱片下载器</h1>
        <span className="text-secondary pt-1">版本 {packageJson.version}</span>
      </div>

      <button onClick={() => openUrl("https://github.com/nuthx/siren-downloader")}>
        <img src="/logo/github.svg" alt="logo" className="size-6 hover:opacity-70 cursor-pointer transition" draggable="false" />
      </button>

      <div className="flex-center flex-col gap-1 text-xs text-secondary text-center">
        <p>本工具仅用于学习与技术交流，所有音乐版权均归原作者及相关版权方所有，任何下载内容不得用于商业用途</p>
        <p>本工具中使用的塞壬唱片及相关 LOGO 均为其合法拥有者的注册商标或版权标识，仅用于标识目的，归相关版权方所有</p>
      </div>
    </div>
  )
}
