import requests
from PIL import Image
from io import BytesIO


def compare_cover(song):
    ncm_cover = requests.get(song["cover_ncm"])
    ncm_width = Image.open(BytesIO(ncm_cover.content)).size[0]

    # 如果网易云的封面小于600px，就用官方的封面
    if ncm_width < 600:
        return requests.get(song["cover"]).content
    else:
        return ncm_cover.content
