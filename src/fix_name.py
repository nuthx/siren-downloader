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
