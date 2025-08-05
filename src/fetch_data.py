import os
import json
import datetime
import requests
import threading

from src.load_config import load_config
from src.check_download import need_download
from src.calc_instrument_num import calc_instrument_num


def fetch_all_songs():
    song_list = {
        "count": 0,
        "instrumental": 0,
        "songs": []
    }

    # 从官网获取歌单
    response = requests.get("https://monster-siren.hypergryph.com/api/songs").json()
    siren_list = response["data"]["list"]

    # 写入歌曲数量
    song_list["count"] = len(siren_list)

    # 写入歌曲信息
    for list in response["data"]["list"]:
        song_list["songs"].append({
            "title": list["name"],
            "title_id": list["cid"],
            "album": "",
            "album_id": list["albumCid"],
            "source": "",
            "cover": "",
            "cover_ncm": "",
            "format": "",
            "track": "",
            "publish": "",
            "instrumental": 0,
            # "download": ""
        })

    # 默认新歌在最前，需要反转排序
    song_list["songs"].reverse()

    # 写入伴奏数量
    song_list["instrumental"] = calc_instrument_num(song_list)

    return song_list


def fetch_album_data(song_list):
    # 只将需要下载的歌曲的专辑加入album_list
    album_list = []
    for song in song_list["songs"]:
        if need_download(song):
            album_list.append({
                "album_id": song["album_id"],
                "album": "",
                "album_ncm": "",
                "cover": ""
            })

    # 去重：先转为元组，再转为列表
    album_list = list({tuple(album.items()) for album in album_list})
    album_list = [dict(album) for album in album_list]

    print(f"待拉取{len(album_list)}条专辑信息")

    # 获取单个专辑信息
    def fetch_album(album):
        url = f"https://monster-siren.hypergryph.com/api/album/{album['album_id']}/detail"
        album_data = requests.get(url).json()
        album["album"] = album_data["data"]["name"].strip()  # 部分专辑名称后有个空格（乌鱼子）
        album["cover"] = album_data["data"]["coverUrl"]

    # 创建线程
    threads = []
    for album in album_list:
        thread = threading.Thread(target=fetch_album, args=(album,))
        threads.append(thread)
        thread.start()

    # 等待线程完成
    for thread in threads:
        thread.join()

    # 读取匹配列表（官网名称：网易云名称）
    with open("conf/match.json", "r", encoding="utf-8") as json_file:
        match_list = json.load(json_file)

    # 根据本地规则，匹配官网名称到网易云名称
    # https://music.163.com/#/artist/album?id=32540734&limit=1000
    for album in album_list:
        if album["album"] in match_list:
            album["album_ncm"] = match_list[album["album"]]
        else:
            album["album_ncm"] = album["album"]

    # 保存到song_list
    for song in song_list["songs"]:
        # 查找album_list对应album_id的字典
        album_dict = next((album for album in album_list if album["album_id"] == song["album_id"]), None)
        if album_dict:
            song["album"] = album_dict["album_ncm"]  # 使用网易云名称作为专辑名
            song["cover"] = album_dict["cover"]

    # 从网易云api获取专辑封面与发布日期
    api_url = load_config("default", "ncm_api")
    ncm_response = requests.get(api_url + "/artist/album?id=32540734&limit=1000").json()
    for song in song_list["songs"]:
        if song.get("album"):  # 只处理需要下载的歌曲
            try:
                ncm_dict = next(album for album in ncm_response["hotAlbums"] if album["name"] == song["album"])
            except StopIteration:
                print(f"跳过：{song['title']}所在的专辑{song['album']}与网易云名称不匹配，请更新匹配规则")
                continue
            song["publish"] = datetime.datetime.fromtimestamp(ncm_dict["publishTime"] / 1000).strftime("%Y-%m-%d")
            song["cover_ncm"] = ncm_dict["picUrl"]


def fetch_song_data(song_list):
    # 只获取需要下载的歌曲信息
    def fetch_song(song):
        if need_download(song):
            url = f"https://monster-siren.hypergryph.com/api/song/{song['title_id']}"
            song_data = requests.get(url).json()
            song["source"] = song_data["data"]["sourceUrl"]
            song["format"] = os.path.splitext(song["source"])[1].replace(".", "")

    # 创建线程
    threads = []
    for song in song_list["songs"]:
        thread = threading.Thread(target=fetch_song, args=(song,))
        threads.append(thread)
        thread.start()

    # 等待线程完成
    for thread in threads:
        thread.join()

    # 专辑内歌曲排序
    track = 0
    current_album = ""
    for song in song_list["songs"]:
        if song["album"] == current_album:
            track += 1
        else:
            current_album = song["album"]
            track = 1
        song["track"] = str(track)
