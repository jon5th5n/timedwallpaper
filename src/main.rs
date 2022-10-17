use std::{process::Command, fs, env};
use chrono::{prelude::*, Duration};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] == "-h" || args[1] == "--help" {
        println!(r#"
create timed wallpapers

USAGE:
    timedwallpaper [configpath] [updatedelay]

INFO:
    configpath: a config file containing timestamps and imagepaths
                imagepaths have to be relative to the config file

CONFIG EXAMPLE:
    00_00 = img1.jpg, img2.jpg
    06_00 = img3.jpg
    12_00 = img4.jpg, img5.jpg, img6.jpg
    21_00 = img7.jpg, img8.jpg
        "#);
        return
    }

    let cfg = parse_config(args[1].as_str());

    let mut working_directory: Vec<&str> = args[1].split('/').collect();
    working_directory.pop();
    let working_directory = working_directory.join("/");

    let update_delay: u64 = args[2].parse().unwrap();

    loop {
        let now = Local::now().time();

        // get filepath of image for this time
        let mut filepath = cfg.last().unwrap().1.as_str();
        for i in 0..cfg.len() {
            if now >= cfg[i].0 {
                continue;
            }

            let n = if i == 0 { cfg.len() - 1 } else { i - 1 };
            filepath = cfg[n].1.as_str();

            break;
        }

        set_wallpaper(format!("{working_directory}/{filepath}").as_str(), "fill");
            
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

fn parse_config(filepath: &str) -> Vec<(NaiveTime, String)> {
    let config = fs::read_to_string(filepath).unwrap().replace(" ", "");

    let config: Vec<&str> = config.split('\n').collect();

    // split entry into timestamp and vec of paths
    let mut config_timestamped: Vec<(NaiveTime, Vec<&str>)> = Vec::new();
    for tp in config {
        let time_paths: Vec<&str> = tp.split('=').collect();

        let time_str: Vec<&str> = time_paths[0].split('_').collect();
        let time = NaiveTime::from_hms(time_str[0].parse().unwrap(), time_str[1].parse().unwrap(), 0);

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
