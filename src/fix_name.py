# 移除文件名中的非法字符
def fix_name(name):
    name = (name
     .replace("  ", " ")
     .replace("<", " ")
     .replace(">", " ")
     .replace(":", " ")
     .replace("\'", "")
     .replace("\"", "")
     .replace("\\", " ")
     .replace("/", " ")
     .replace("|", " ")
     .replace("?", "")
     .replace("*", " "))
    return name


# 移除文件夹末尾的点号
def fix_folder(name):
    name = fix_name(name).rstrip(".")
    return name
