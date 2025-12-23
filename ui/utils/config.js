import { load } from "@tauri-apps/plugin-store"

export const DEFAULT_CONFIG = {
  download_path: "",
  download_instrumental: false,
  download_lyrics: false,
  show_cover: true
}

export const SYSTEM_ALBUM_MATCH = {
  "Operation Barrenland": "Operation Barrenland (W&W Soundtrack Mix)",
  "危机合约黄铁·利刃·燃灰OST": "危机合约 黄铁·利刃·燃灰OST",
  "Operation Pine Soot": "危机合约松烟行动OST",
  "2022明日方舟音律联觉-灯下定影原声EP": "音律联觉-灯下定影原声EP",
  "8-bit弹雨与断罪之拳": "狂弹要塞！罗德大兵集结",
  "冉冉升起，直播新星": "最后的全能系美少女！主播U的每日泰拉分享 (真没跑路)",
  "冬隐归路OST": "明日方舟：冬隐归路OST",
  "A Toda Vela": "出苍白海OST",
  "焰烬曙明OST": "明日方舟：焰烬曙明OST",
  "卫戍协议盟约OST": "卫戍协议：盟约OST"
}

let storeInstance = null

async function getConfigStore() {
  if (!storeInstance) {
    storeInstance = await load("config.json", { autoSave: false })
  }
  return storeInstance
}

export async function initConfig() {
  const store = await getConfigStore()
  await store.set("app_config", (await store.get("app_config")) || DEFAULT_CONFIG)
  await store.set("album_match", SYSTEM_ALBUM_MATCH)
  await store.save()
}

export async function getConfig() {
  const store = await getConfigStore()
  const config = await store.get("app_config")
  return config || DEFAULT_CONFIG
}

export async function saveConfig(config) {
  const store = await getConfigStore()
  await store.set("app_config", config)
  await store.save()
}
