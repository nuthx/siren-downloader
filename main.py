import os
import json
import datetime
import requests
import threading

from src.config import load_config
from src.folder import new_folder
from src.download import download_music


def get_hypergryph_playlist():
    # 从官网获取歌单
    response = requests.get("https://monster-siren.hypergryph.com/api/songs").json()
    song_list = response["data"]["list"]
    song_list.reverse()  # 默认新歌在最前，翻转顺序
    song_count_all = len(song_list)

    # 移除伴奏
    song_list = [song for song in song_list if "instrument" not in song["name"].lower()]
    song_count = len(song_list)

    print(f"移除了{song_count_all - song_count}首伴奏，还剩{song_count}首音乐")
    return song_list


def get_hypergryph_music_data(this_song):
    # 请求专辑数据
    album_id = this_song["albumCid"]
    album_url = f"https://monster-siren.hypergryph.com/api/album/{album_id}/detail"
    album_data = requests.get(album_url).json()

    # 请求歌曲数据
    song_id = this_song["cid"]
    song_url = f"https://monster-siren.hypergryph.com/api/song/{song_id}"
    song_data = requests.get(song_url).json()

    # 写入专辑名称与封面地址
    this_song["name"] = this_song["name"].strip()
    this_song["name_alt"] = this_song["name"].replace(":", " ").replace("  ", " ").replace("\'", "")
    this_song["albumName"] = album_data["data"]["name"].strip().replace("2022明日方舟", "")
    this_song["sourceUrl"] = song_data["data"]["sourceUrl"]
    this_song["coverUrl"] = album_data["data"]["coverUrl"]
    this_song["coverName"] = os.path.basename(this_song["coverUrl"])
    this_song["ext"] = os.path.splitext(this_song["sourceUrl"])[1].replace(".", "")


def add_track_number(song_list):
    track = 0
    current_album = ""
    for song in song_list:
        if song["albumName"] == current_album:
            track += 1
        else:
            current_album = song["albumName"]
            track = 1
        song["track"] = str(track)


def get_ncm_cover_and_year(song_list):
    # 专辑地址
    # https://music.163.com/#/artist/album?id=32540734&limit=1000
    ncm_list = requests.get(load_config("api", "netease") + "/artist/album?id=32540734&limit=1000").json()
    for song in song_list:
        for album in ncm_list["hotAlbums"]:

            # 专辑名称匹配
            if song["albumName"] == "Operation Barrenland":
                album_name_ncm = "Operation Barrenland (W&W Soundtrack Mix)"
            elif song["albumName"] == "危机合约黄铁·利刃·燃灰OST":
                album_name_ncm = "危机合约 黄铁·利刃·燃灰OST"
            elif song["albumName"] == "Operation Pine Soot":
                album_name_ncm = "危机合约松烟行动OST"
            elif song["albumName"] == "8-bit弹雨与断罪之拳":
                album_name_ncm = "狂弹要塞！罗德大兵集结"
            elif song["albumName"] == "冉冉升起，直播新星":
                album_name_ncm = "最后的全能系美少女！主播U的每日泰拉分享 (真没跑路)"
            elif song["albumName"] == "冬隐归路OST":
                album_name_ncm = "明日方舟：冬隐归路OST"
            else:
                album_name_ncm = song["albumName"]

            # 获取网易云结果
            if album_name_ncm == album["name"]:
                song["publish"] = datetime.datetime.fromtimestamp(album["publishTime"]/1000).strftime("%Y-%m-%d")
                song["coverUrl_ncm"] = album["picUrl"]
                break
    return


def write_to_json(name, file_format):
    with open("conf/download.json", "r", encoding="utf-8") as json_file:
        data = json.load(json_file)

    data[name] = file_format

    with open("conf/download.json", "w", encoding="utf-8") as json_file:
        json.dump(data, json_file, ensure_ascii=False, indent=4)


if __name__ == "__main__":
    # 拉取官网的歌曲信息
    print("——————————")
    print("获取专辑列表")
    song_list = get_hypergryph_playlist()

    # 创建线程，在song_list写入歌曲数据
    print("——————————")
    print("获取歌曲数据")
    threads = []
    for song in song_list:
        thread = threading.Thread(target=get_hypergryph_music_data, args=(song,))
        threads.append(thread)
        thread.start()

    # 等待线程完成
    for thread in threads:
        thread.join()

    # 添加曲目号
    print("整理曲目编号")
    add_track_number(song_list)

    # 从网易云API获取封面与专辑年份，写入song_list
    print("获取封面与年份")
    get_ncm_cover_and_year(song_list)

    # 确认下载路径
    print("——————————")
    new_folder(load_config("path", "download"))

    # 读取download.json内容到download_list
    # 若不存在，则生成空的download_list
    print("读取已下载的歌曲信息")
    if os.path.exists("conf/download.json"):
        with open("conf/download.json", "r", encoding="utf-8") as json_file:
            download_list = json.load(json_file)
    else:
        download_list = {}

    # 保存歌曲总数
    download_list["all_music"] = len(song_list)

    # 保存歌曲信息，不存在则新建一条
    for song in song_list:
        song_comb = song["albumName"] + "/" + song["name"]
        if song_comb not in download_list:
            download_list[song_comb] = "pending"

    # 写入新增音乐到download.json
    print("写入下载状态到本地")
    with open("conf/download.json", "w", encoding="utf-8") as file:
        json.dump(download_list, file, ensure_ascii=False, indent=4)

    # 开始下载
    print("——————————")
    for song in song_list:
        song_comb = song["albumName"] + "/" + song["name"]

        # 跳过已下载
        if download_list[song_comb] != "pending":
            continue

        # 下载歌曲，并写入到json
        ext = download_music(song)

        # 写入到json
        write_to_json(song_comb, ext)
        print("下载完成：" + song["name_alt"])
        print("—————")
