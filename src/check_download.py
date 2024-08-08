from src.load_config import load_config


def need_download(song):
    # 已下载
    if "download" in song:
        return False

    # 伴奏
    if load_config("default", "skip_instrumental", bool=True) and song["instrumental"] == 1:
        return False

    return True
