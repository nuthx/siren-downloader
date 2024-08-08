import os


def check_folder(name):
    if os.path.exists(name):
        return True
    else:
        os.mkdir(name)
        return False
