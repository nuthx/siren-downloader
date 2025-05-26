import requests
from PIL import Image
from io import BytesIO


def best_cover(song):
    ncm_cover = requests.get(song["cover_ncm"])
    img = Image.open(BytesIO(ncm_cover.content))
    width, height = img.size

    # 如果网易云的封面小于600px，就用官方的封面
    if width < 600 or height < 600:
        return requests.get(song["cover"]).content
    
    # 如果网易云的封面大于2000px，就压缩至2000px
    if width > 2000 or height > 2000:
        # 计算新的尺寸，保持比例
        ratio = width / height
        if width > height:
            new_width = 2000
            new_height = int(2000 / ratio)
        else:
            new_height = 2000
            new_width = int(2000 * ratio)
        
        # 将压缩后的图片转换为bytes
        img = img.resize((new_width, new_height), Image.Resampling.LANCZOS)
        buffer = BytesIO()
        img.save(buffer, format="PNG", optimize=True)
        return buffer.getvalue()
    
    return ncm_cover.content
