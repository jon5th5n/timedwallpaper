# timedwallpaper
a way to create timed wallpapers using rust as the language and feh to set the wallpaper

### BUILD
    cargo build -r

### USAGE:
    timedwallpaper [OPTIONS] --folder <FOLDER>

### OPTIONS
    -f, --folder <FOLDER>
        path to a folder containing:
        - wallpaper.config
            file containing time-wallpaper relationship
            !IMPORTANT! time shortcuts have to be followed with :[defaulttime]
        - data.ini
            file containing additional data for extra functionality

    -d, --delay <DELAY>
        number of seconds to wait between updates
        
        [default: 600]

    -h, --help
        Print help information (use `-h` for a summary)

    -V, --version
        Print version information

### INFO:
    time shortcuts:
        - #tb -> twilight begin
        - #sr -> sunrise
        - #sn -> solar noon
        - #ss -> sunset
        - #te -> twilight end
        - #sm -> solar midnight

### CONFIG EXAMPLE:
    wallpaper.config:
        00_00 = img1.jpg, img2.jpg
        #sr:06_00 = img3.jpg
        12_00 = img4.jpg, img5.jpg, img6.jpg
        #ss:21_00 = img7.jpg, img8.jpg
    
    data.ini:
        [Sun]
        latitude = 69.187
        longitude = 1.02
