import os


def check_folder(name):
    path = os.path.expanduser(name)
    if os.path.exists(path):
        return True
    else:
        os.makedirs(path, exist_ok=True)
        return False
