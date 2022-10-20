use std::{process::Command, fs, env, collections::HashMap};
use chrono::{prelude::*, Duration};
use serde_json::{Map, Value};
use ini::Ini;

const SUN_DATA: &'static [&'static str] = &["civil_twilight_begin", "sunrise", "solar_noon", "sunset", "civil_twilight_end", "solar_midnight"];
fn sun_data_key_from_shortcut(sc: &str) -> &str {
    match sc {
        "tb" => "civil_twilight_begin",
        "sr" => "sunrise",
        "sn" => "solar_noon",
        "ss" => "sunset",
        "te" => "civil_twilight_end",
        "sm" => "solar_midnight",
        _ => panic!("sun data shortcut is not recognized")
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] == "-h" || args[1] == "--help" {
        println!(r#"
create timed wallpapers

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
        "#);
        return
    }

    //-- init ----
    let working_directory = args[1].clone();
    let update_delay: u64 = args[2].parse().unwrap();

    let is_online = online::check(None).is_ok();
    let mut current_wallpaper: String = "".to_string();

    let config_data = Ini::load_from_file(format!("{working_directory}/data.ini").as_str(),).unwrap();

    let cycle;
    // get sun data if online and parse cycle config
    if is_online {
        let config_sun = config_data.section(Some("Sun")).unwrap();
        let lat = config_sun.get("latitude").unwrap().parse::<f32>().unwrap();
        let lng = config_sun.get("longitude").unwrap().parse::<f32>().unwrap();
        let sun_data = get_sun_data(lat, lng);
        cycle = parse_cycle_online(format!("{working_directory}/wallpaper.config").as_str(), sun_data);
    } else {
        cycle = parse_cycle_offline(format!("{working_directory}/wallpaper.config").as_str(),);
    }

    //-- loop ---
    loop {
        let now = Local::now().time();

        // get filepath of image for this time
        let mut filepath = cycle.last().unwrap().1.as_str();
        for i in 0..cycle.len() {
            if now >= cycle[i].0 {
                continue;
            }

            let n = if i == 0 { cycle.len() - 1 } else { i - 1 };
            filepath = cycle[n].1.as_str();

            break;
        }

        if format!("{working_directory}/{filepath}") != current_wallpaper {
            set_wallpaper(format!("{working_directory}/{filepath}").as_str(), "fill");
            current_wallpaper = format!("{working_directory}/{filepath}");
        }
            
        std::thread::sleep(std::time::Duration::from_secs(update_delay));
    }
}

fn set_wallpaper(filepath: &str, option: &str) {
    let out = Command::new("sh")
            .arg("-c")
            .arg(format!("feh --bg-{option} {filepath}"))
            .output()
            .expect("failed to execute process");
    
    if out.stderr.len() != 0 {
        println!("{}", std::str::from_utf8(&out.stderr).unwrap());
        return
    }
    println!("changed wallpaper to {filepath}");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ApiData {
    results: Map<String, Value>,
    status: String,
}

fn get_sun_data(lat: f32, lng: f32) -> HashMap<String, NaiveTime> {

    let res_string = reqwest::blocking::get(format!("https://api.sunrise-sunset.org/json?lat={lat}&lng={lng}")).unwrap().text().unwrap();
    let res: ApiData = serde_json::from_str(res_string.as_str()).unwrap();
    let res = res.results;

    let mut results: HashMap<String, NaiveTime> = HashMap::new();
    for (k, v) in res {
        if !SUN_DATA.contains(&k.as_str()) { continue };

        let s = v.as_str().unwrap();

        let tmp: Vec<&str> = s.split(' ').collect();
        let (time, pm) = (tmp[0], tmp[1]);
        let pm = if pm == "PM" { true } else { false };

        let hms: Vec<&str> = time.split(':').collect();
        let (h, m, s) = (hms[0].parse().unwrap(), hms[1].parse().unwrap(), hms[2].parse().unwrap());
        let mut time = NaiveTime::from_hms(h, m, s);
        if pm {
            time += Duration::hours(12);
        }

        let timezone_offset = Local::now().offset().to_string();
        let timezone_offset: Vec<&str> = timezone_offset.get(1..).unwrap().split(':').collect();
        time += Duration::minutes(60 * timezone_offset[0].parse::<i64>().unwrap() + timezone_offset[1].parse::<i64>().unwrap());

        results.insert(k, time);
    }

    // add solar_midnight (solar_noon + 12h)
    results.insert("solar_midnight".to_string(), results.get("solar_noon").unwrap().clone() + Duration::hours(12));

    results
}

fn parse_cycle_online(filepath: &str, sun_data: HashMap<String, NaiveTime>) -> Vec<(NaiveTime, String)> {
    let config = fs::read_to_string(filepath).unwrap().replace(" ", "");

    let config: Vec<&str> = config.split('\n').collect();

    // split entry into timestamp and vec of paths
    let mut config_timestamped: Vec<(NaiveTime, Vec<&str>)> = Vec::new();
    for tp in config {
        let time_paths: Vec<&str> = tp.split('=').collect();

        let mut time: NaiveTime;
        if time_paths[0].starts_with("#") {
            let time_str: Vec<&str> = time_paths[0].split(':').collect();
            let shortcut = time_str[0].get(1..).unwrap();
            time = sun_data.get(sun_data_key_from_shortcut(shortcut)).unwrap().clone();
        } else {
            let time_str: Vec<&str> = time_paths[0].split('_').collect();
            time = NaiveTime::from_hms(time_str[0].parse().unwrap(), time_str[1].parse().unwrap(), 0);
        }

        let paths: Vec<&str> = time_paths[1].split(',').collect();
        config_timestamped.push((time, paths));
    }

    // sort after timestamps
    config_timestamped.sort();

    // create interpolated timestamps for paths between one and another timestamp
    let mut config_interpolated: Vec<(NaiveTime, String)> = Vec::new();
    for i in 0..config_timestamped.len() {
        let (time, paths) = &config_timestamped[i];

        let interpolation_step = if i == config_timestamped.len() - 1 { 
            ((config_timestamped[0].0 - *time) + Duration::minutes(1440)) / (config_timestamped.len() - 1) as i32
        } else {
            (config_timestamped[i+1].0 - *time) / (config_timestamped.len() - 1) as i32
        };

        for j in 0..paths.len() {
            config_interpolated.push((*time + interpolation_step * j as i32, paths[j].to_string()));
        }
    }

    config_interpolated
}

fn parse_cycle_offline(filepath: &str) -> Vec<(NaiveTime, String)> {
    let config = fs::read_to_string(filepath).unwrap().replace(" ", "");

    let config: Vec<&str> = config.split('\n').collect();

    // split entry into timestamp and vec of paths
    let mut config_timestamped: Vec<(NaiveTime, Vec<&str>)> = Vec::new();
    for tp in config {
        let time_paths: Vec<&str> = tp.split('=').collect();

        let mut time: NaiveTime;

        let time_str: Vec<&str> = if time_paths[0].starts_with("#") {
            time_paths[0].split(':').collect::<Vec<&str>>()[1].split('_').collect()
        } else {
            time_paths[0].split('_').collect()
        };

        time = NaiveTime::from_hms(time_str[0].parse().unwrap(), time_str[1].parse().unwrap(), 0);

        let paths: Vec<&str> = time_paths[1].split(',').collect();
        config_timestamped.push((time, paths));
    }

    // sort after timestamps
    config_timestamped.sort();

    // create interpolated timestamps for paths between one and another timestamp
    let mut config_interpolated: Vec<(NaiveTime, String)> = Vec::new();
    for i in 0..config_timestamped.len() {
        let (time, paths) = &config_timestamped[i];

        let interpolation_step = if i == config_timestamped.len() - 1 { 
            ((config_timestamped[0].0 - *time) + Duration::minutes(1440)) / (config_timestamped.len() - 1) as i32
        } else {
            (config_timestamped[i+1].0 - *time) / (config_timestamped.len() - 1) as i32
        };

        for j in 0..paths.len() {
            config_interpolated.push((*time + interpolation_step * j as i32, paths[j].to_string()));
        }
    }

    config_interpolated
}