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

    # 更新专辑数据：全部
    if not load_config("debug", "skip_album_update", bool=True):
        print(f"更新专辑数据")
        fetch_album_data(song_list_local)

    # 更新歌曲数据：仅需要下载
    if not load_config("debug", "skip_song_update", bool=True):
        print("更新歌曲数据")
        fetch_song_data(song_list_local)

    print("保存数据到本地")
    with open("conf/data.json", "w", encoding="utf-8") as json_file:
        json.dump(song_list_local, json_file, ensure_ascii=False, indent=4)

    return song_list_local
