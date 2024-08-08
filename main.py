import json

from src.check_download import need_download
from src.load_config import load_config
from src.check_folder import check_folder
from src.download_music import download_music
from src.fetch_data import fetch_all_songs
from src.update_data import update_data


if __name__ == "__main__":
    print("明日方舟塞壬唱片专辑下载器")
    print("版本 2.0.0")

    # 拉取官网的歌曲信息
    print("——————————")
    print("获取在线专辑")
    song_list = fetch_all_songs()
    print(f"共{song_list['count']}首音乐，其中{song_list['instrumental']}首伴奏")

    # 通过歌曲数量，检查本地是否最新
    print("——————————")
    song_list = update_data(song_list)

    # 确认下载路径
    if not check_folder(load_config("default", "download_path")):
        print("——————————")
        print("新建文件夹：" + load_config("default", "download_path"))

    # 开始下载
    print("——————————")
    for song in song_list["songs"]:
        # 跳过已下载或伴奏
        if not need_download(song):
            continue

        # 每下载完成一首，写入到本地json
        song["download"] = download_music(song)
        with open("conf/data.json", "w", encoding="utf-8") as json_file:
            json.dump(song_list, json_file, ensure_ascii=False, indent=4)
        print("下载完成：" + song["title"])
        print("—————")
