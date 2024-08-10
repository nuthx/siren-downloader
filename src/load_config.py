import configparser


def load_config(category, item, bool=False):
    config = configparser.ConfigParser()
    with open("conf/config.ini", "r", encoding="utf-8") as config_file:
        config.read_file(config_file)

    if bool:
        return config.getboolean(category, item)
    else:
        return config.get(category, item)
