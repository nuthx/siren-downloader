import os
import tqdm
import requests
import send2trash
from PIL import Image
from io import BytesIO
from mutagen.flac import FLAC, Picture
from mutagen.id3 import ID3, TIT2, TPE1, TRCK, TALB, TDRC, TCON, TPE2, TPOS, APIC

from src.config import load_config
from src.folder import new_folder


def download_music(song):
    album_name = song["albumName"]
    song_name = song["name_alt"]  # 使用调整后的文件名，避免特殊符号

    album_path = os.path.join(load_config("path", "download"), album_name)
    song_path = os.path.join(album_path, song_name) + "." + song["ext"]
    flac_path = os.path.join(album_path, song_name) + ".flac"

    # 开始下载
    new_folder(album_path)
    print(f"开始下载：{song_name}.{song['ext']}")

    # 显示进度条
    song_file = requests.get(song["sourceUrl"], stream=True)
    file_size = int(song_file.headers.get('content-length', 0))
    with open(song_path, "wb") as file, tqdm.tqdm(
        desc=f"正在下载：{song_name}.{song['ext']}",
        total=file_size,
        unit='B',
        unit_scale=True,
        unit_divisor=1024,
    ) as bar:
        for data in song_file.iter_content(chunk_size=1024):
            file.write(data)
            bar.update(len(data))

    # 转码
    if song["ext"] == "wav":
        print(f"开始转码：{song_name}")
        command = f"ffmpeg -i '{song_path}' -codec:a flac -level 0 -y '{flac_path}' -loglevel quiet"
        os.system(command)
        send2trash.send2trash(song_path)
    elif song["ext"] == "mp3":
        print(f"无需转码：{song_name}")

    # 写入ID3
    print(f"写入标签：{song_name}")
    if song["ext"] == "wav":
        flac = FLAC(flac_path)
        flac["title"] = song["name"]
        flac["artist"] = "塞壬唱片-MSR"
        flac["albumartist"] = "塞壬唱片-MSR"
        flac["tracknumber"] = song["track"]
        flac["album"] = song["albumName"]
        flac["discnumber"] = "1"
        flac["date"] = song["publish"][:4]
        flac["genre"] = "Arknights"

        # 写入封面
        cover = Picture()
        cover.type = 3
        cover.mime = 'image/jpeg'
        cover.data = bigger_cover(song)
        flac.clear_pictures()
        flac.add_picture(cover)
        flac.save()

        return "flac"

    elif song["ext"] == "mp3":
        mp3 = ID3(song_path)
        mp3["TIT2"] = TIT2(encoding=3, text=song["name"])
        mp3["TPE1"] = TPE1(encoding=3, text="塞壬唱片-MSR")
        mp3["TPE2"] = TPE2(encoding=3, text="塞壬唱片-MSR")
        mp3["TRCK"] = TRCK(encoding=3, text=song["track"])
        mp3["TALB"] = TALB(encoding=3, text=song["albumName"])
        mp3["TPOS"] = TPOS(encoding=3, text="1")
        mp3["TDRC"] = TDRC(encoding=3, text=song["publish"][:4])
        mp3["TCON"] = TCON(encoding=3, text="Arknights")

        # 写入封面
        cover_data = bigger_cover(song)
        apic = APIC(encoding=3, mime="image/jpeg", type=3, data=cover_data)
        mp3.add(apic)
        mp3.save()

        return "mp3"


def bigger_cover(song):
    ncm_cover = requests.get(song["coverUrl_ncm"])
    ncm_width = Image.open(BytesIO(ncm_cover.content)).size[0]

    # 如果网易云的封面小于600px，就用官方的封面
    if ncm_width < 600:
        return requests.get(song["coverUrl"]).content
    else:
        return ncm_cover.content
