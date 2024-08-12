import os
import json

from src.load_config import load_config
from src.fetch_data import fetch_album_data, fetch_song_data


def update_data(song_list):
    # 本地是否有data.json
    if os.path.exists("conf/data.json"):
        with open("conf/data.json", "r", encoding="utf-8") as json_file:
            song_list_local = json.load(json_file)
    else:
        song_list_local = song_list

    # 是否有新增歌曲
    len_local = len(song_list_local["songs"])
    len_new = len(song_list["songs"])
    if len_new > len_local:
        missing = song_list["songs"][len_local:]
        song_list_local["songs"].extend(missing)
        print(f"发现{len(missing)}首新增音乐")

    # 更新专辑数据：全部
    if not load_config("debug", "skip_album_update", bool=True):
        print(f"正在更新专辑数据")
        fetch_album_data(song_list_local)

    # 更新歌曲数据：仅需要下载
    if not load_config("debug", "skip_song_update", bool=True):
        print("正在更新歌曲数据")
        fetch_song_data(song_list_local)

    # 保存到data.json
    print("成功保存数据到本地")
    with open("conf/data.json", "w", encoding="utf-8") as json_file:
        json.dump(song_list_local, json_file, ensure_ascii=False, indent=4)

    return song_list_local
