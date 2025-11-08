import os
import json
import requests
import threading
from mutagen.id3 import ID3, USLT

from src.load_config import load_config
from src.fix_name import fix_name, fix_folder


def update_lyric_urls(song_list):
    """从API获取歌词URL并更新到song_list"""
    
    print("🔄 正在从API获取歌词URL...")
    
    # 需要更新的歌曲（已下载的）
    songs_to_update = [song for song in song_list["songs"] if song.get("download")]
    
    if not songs_to_update:
        print("  ⚠️  没有已下载的歌曲需要更新")
        return
    
    print(f"  📝 需要更新 {len(songs_to_update)} 首歌曲的歌词URL")
    
    # 获取单个歌曲的歌词URL
    def fetch_lyric_url(song):
        try:
            url = f"https://monster-siren.hypergryph.com/api/song/{song['title_id']}"
            song_data = requests.get(url, timeout=10).json()
            # 安全获取歌词URL
            song["lyric"] = song_data["data"].get("lyricUrl", "")
            if song["lyric"]:
                print(f"  ✅ {song['title']}")
            else:
                print(f"  ⚠️  {song['title']} - 无歌词")
        except Exception as e:
            print(f"  ❌ {song['title']} - 获取失败: {e}")
            song["lyric"] = ""
    
    # 创建线程批量获取
    threads = []
    for song in songs_to_update:
        thread = threading.Thread(target=fetch_lyric_url, args=(song,))
        threads.append(thread)
        thread.start()
    
    # 等待所有线程完成
    for thread in threads:
        thread.join()
    
    print("  ✅ 歌词URL更新完成")


def download_lyric_for_existing(song):
    """为已下载的歌曲补充歌词"""
    
    # 检查是否已下载
    if not song.get("download"):
        return False
    
    # 检查是否有歌词URL
    if not song.get("lyric"):
        return False
    
    album_name = song["album"]
    song_name = song["title"]
    lyric_name = song["title"] + ".lrc"
    
    # 构建文件路径
    expanded_path = os.path.expanduser(load_config("default", "download_path"))
    album_path = os.path.join(expanded_path, fix_folder(album_name))
    lyric_path = os.path.join(album_path, fix_name(lyric_name))
    
    # 获取音乐文件路径（根据download字段判断格式）
    file_format = song["download"]
    if file_format == "flac":
        music_path = os.path.join(album_path, fix_name(song_name)) + ".flac"
    elif file_format == "mp3":
        music_path = os.path.join(album_path, fix_name(song_name)) + ".mp3"
    else:
        print(f"  ⚠️  未知格式：{file_format}")
        return False
    
    # 检查音乐文件是否存在
    if not os.path.exists(music_path):
        print(f"  ⚠️  音乐文件不存在：{music_path}")
        return False
    
    # 检查lrc文件是否已存在
    lrc_exists = os.path.exists(lyric_path)
    
    # 下载歌词文件
    lyric_content = ""
    try:
        print(f"  📥 下载歌词：{song_name}")
        lyric_file = requests.get(song["lyric"], timeout=10)
        lyric_file.raise_for_status()
        
        # 保存lrc文件
        if not lrc_exists:
            with open(lyric_path, "wb") as file:
                file.write(lyric_file.content)
            print(f"  ✅ LRC文件保存成功")
        else:
            print(f"  ⏭️  LRC文件已存在，跳过")
        
        # 获取歌词内容
        lyric_content = lyric_file.content.decode('utf-8')
        
    except Exception as e:
        print(f"  ❌ 歌词下载失败：{e}")
        return False
    
    # 如果是mp3格式，写入ID3标签
    if file_format == "mp3" and lyric_content:
        try:
            mp3 = ID3(music_path)
            
            # 检查是否已有歌词标签
            if "USLT::chi" in mp3:
                print(f"  ⏭️  歌词已存在于ID3标签中")
            else:
                mp3["USLT"] = USLT(encoding=3, lang='chi', desc='', text=lyric_content)
                mp3.save()
                print(f"  ✅ 歌词已写入MP3的ID3标签")
        except Exception as e:
            print(f"  ❌ 写入MP3标签失败：{e}")
            return False
    
    return True


if __name__ == "__main__":
    print("=" * 50)
    print("🎵 塞壬唱片歌词补充工具")
    print("=" * 50)
    
    # 读取data.json
    if not os.path.exists("conf/data.json"):
        print("❌ 未找到 conf/data.json 文件")
        exit(1)
    
    with open("conf/data.json", "r", encoding="utf-8") as json_file:
        song_list = json.load(json_file)
    
    print(f"📚 共找到 {song_list['count']} 首歌曲")
    
    # 筛选已下载的歌曲
    downloaded_songs = [song for song in song_list["songs"] if song.get("download")]
    print(f"📥 其中已下载 {len(downloaded_songs)} 首")
    print("=" * 50)
    
    # 更新歌词URL字段
    update_lyric_urls(song_list)
    
    # 保存更新后的data.json
    print("\n💾 保存更新后的 data.json...")
    with open("conf/data.json", "w", encoding="utf-8") as json_file:
        json.dump(song_list, json_file, ensure_ascii=False, indent=4)
    print("  ✅ 保存成功")
    
    # 筛选有歌词的歌曲
    songs_with_lyric = [song for song in downloaded_songs if song.get("lyric")]
    print(f"\n🎤 共有 {len(songs_with_lyric)} 首歌曲有歌词")
    print("=" * 50)
    
    if not songs_with_lyric:
        print("✨ 没有需要处理的歌曲")
        exit(0)
    
    # 处理每首歌
    success_count = 0
    skip_count = 0
    fail_count = 0
    
    for i, song in enumerate(songs_with_lyric, 1):
        print(f"\n[{i}/{len(songs_with_lyric)}] {song['title']} - {song['album']}")
        
        result = download_lyric_for_existing(song)
        
        if result:
            success_count += 1
        else:
            fail_count += 1
    
    # 显示统计信息
    print("\n" + "=" * 50)
    print("📊 处理完成！")
    print(f"  ✅ 成功：{success_count} 首")
    print(f"  ❌ 失败：{fail_count} 首")
    print("=" * 50)
