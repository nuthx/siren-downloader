import os
import json

from src.fetch_data import fetch_album_data, fetch_song_data


def update_data(song_list):
    # 本地是否有data.json
    if os.path.exists("conf/data.json"):
        with open("conf/data.json", "r", encoding="utf-8") as json_file:
            song_list_local = json.load(json_file)
    else:
        song_list_local = song_list

    # 更新全部
    print(f"更新专辑数据")
    fetch_album_data(song_list_local)

    # 只更新需要下载的
    print("更新歌曲数据")
    fetch_song_data(song_list_local)

    print("保存数据到本地")
    with open("conf/data.json", "w", encoding="utf-8") as json_file:
        json.dump(song_list_local, json_file, ensure_ascii=False, indent=4)

    return song_list_local
