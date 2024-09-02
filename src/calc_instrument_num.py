
def calc_instrument_num(song_list):
    instrumental = 0
    for song in song_list["songs"]:
        if "instrument" in song["title"].lower():
            song["instrumental"] = 1
            instrumental += 1

    return instrumental
