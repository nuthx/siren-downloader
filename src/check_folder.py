import os


def check_folder(name):
    if os.path.exists(os.path.expanduser(name)):
        return True
    else:
        os.mkdir(os.path.expanduser(name))
        return False
