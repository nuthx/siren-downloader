import configparser


def load_config(category, item, bool=False):
    config = configparser.ConfigParser()
    config.read("conf/config.ini")

    if bool:
        return config.getboolean(category, item)
    else:
        return config.get(category, item)
