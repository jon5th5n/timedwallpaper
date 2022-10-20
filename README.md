# timedwallpaper
a way to create timed wallpapers using rust as the language and feh to set the wallpaper

USAGE:
    timedwallpaper [configfolder] [updatedelay]

INFO:
    [configfolder]: a folder containing the files:
                    - wallpaper.config
                      file containing time-wallpaper relationship
                      ! time shortcuts have to be followed with :[defaulttime]
                    - data.ini
                      file containing additional data for extra functionality
    
    [updatedelay]: number of seconds to wait between updates
    
    time shortcuts:
        - #tb -> twilight begin
        - #sr -> sunrise
        - #sn -> solar noon
        - #ss -> sunset
        - #te -> twilight end
        - #sm -> solar midnight

CONFIG EXAMPLE:
    wallpaper.config:
        00_00 = img1.jpg, img2.jpg
        #sr:06_00 = img3.jpg
        12_00 = img4.jpg, img5.jpg, img6.jpg
        #ss:21_00 = img7.jpg, img8.jpg
    
    data.ini:
        [Sun]
        latitude = 69.187
        longitude = 1.02
