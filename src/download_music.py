import os
import tqdm
import requests
import send2trash
from mutagen.flac import FLAC, Picture
from mutagen.id3 import ID3, TIT2, TPE1, TRCK, TALB, TDRC, TCON, TPE2, TPOS, APIC

from src.load_config import load_config
from src.check_folder import check_folder
from src.compare_cover import compare_cover
from src.fix_name import fix_name, fix_folder


def download_music(song):
    album_name = song["album"]
    song_name = song["title"]
    album_name = song["album"]

    expanded_path = os.path.expanduser(load_config("default", "download_path"))
    album_path = os.path.join(expanded_path, fix_folder(album_name))
    song_path = os.path.join(album_path, fix_name(song_name)) + "." + song["format"]
    flac_path = os.path.join(album_path, fix_name(song_name)) + ".flac"

    # 开始下载
    check_folder(album_path)
    print(f"开始下载：{song_name}.{song['format']}")

    # 显示进度条
    song_file = requests.get(song["source"], stream=True)
    file_size = int(song_file.headers.get('content-length', 0))
    with open(song_path, "wb") as file, tqdm.tqdm(
        desc=f"正在下载：{song_name} [{album_name}]",
        total=file_size,
        unit='B',
        unit_scale=True,
        unit_divisor=1024,
    ) as bar:
        for data in song_file.iter_content(chunk_size=1024):
            file.write(data)
            bar.update(len(data))

    # 转码
    if song["format"] == "wav":
        print(f"开始转码：{song_name}")
        command = f'ffmpeg -i "{song_path}" -codec:a flac -level 0 -y "{flac_path}" -loglevel error'
        os.system(command)
        send2trash.send2trash(song_path)

    # 写入ID3
    print(f"写入标签：{song_name}")
    if song["format"] == "wav":
        flac = FLAC(flac_path)
        flac["title"] = song["title"]
        flac["artist"] = "塞壬唱片-MSR"
        flac["albumartist"] = "塞壬唱片-MSR"
        flac["tracknumber"] = song["track"]
        flac["album"] = song["album"]
        flac["discnumber"] = "1"
        flac["date"] = song["publish"][:4]
        flac["genre"] = "Arknights"

        # 写入封面
        cover = Picture()
        cover.type = 3
        cover.mime = 'image/jpeg'
        cover.data = compare_cover(song)
        flac.clear_pictures()
        flac.add_picture(cover)
        flac.save()

        return "flac"

    elif song["format"] == "mp3":
        mp3 = ID3(song_path)
        mp3["TIT2"] = TIT2(encoding=3, text=song["title"])
        mp3["TPE1"] = TPE1(encoding=3, text="塞壬唱片-MSR")
        mp3["TPE2"] = TPE2(encoding=3, text="塞壬唱片-MSR")
        mp3["TRCK"] = TRCK(encoding=3, text=song["track"])
        mp3["TALB"] = TALB(encoding=3, text=song["album"])
        mp3["TPOS"] = TPOS(encoding=3, text="1")
        mp3["TDRC"] = TDRC(encoding=3, text=song["publish"][:4])
        mp3["TCON"] = TCON(encoding=3, text="Arknights")

        # 写入封面
        cover_data = compare_cover(song)
        apic = APIC(encoding=3, mime="image/jpeg", type=3, data=cover_data)
        mp3.add(apic)
        mp3.save()

        return "mp3"
