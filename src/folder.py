import os


def new_folder(name):
    if not os.path.exists(name):
        os.mkdir(name)
        print(f"创建文件夹：{name}")
