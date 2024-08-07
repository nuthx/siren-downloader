import configparser


def load_config(category, item):
    config = configparser.ConfigParser()
    config.read("conf/config.ini")
    return config.get(category, item)
