mod parseconfigs;
use crate::parseconfigs::*;

mod sundata;
use crate::sundata::*;

use std::{process::Command, fs, env};
use chrono::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] == "-h" || args[1] == "--help" {
        println!("{}", fs::read_to_string("./assets/help.txt").unwrap());
        return
    }

    //-- init ----
    let working_directory = args[1].clone();
    let update_delay: u64 = args[2].parse().unwrap();

    let is_online = online::check(None).is_ok();
    let mut current_wallpaper: String = "".to_string();

    let additional_data = parse_data(format!("{working_directory}/data.ini").as_str());

    let mut cycle;
    // get sun data if online and parse cycle config
    if is_online {
        let sun_data = get_sun_data(additional_data.lat, additional_data.lng);
        cycle = parse_cycle_online(format!("{working_directory}/wallpaper.config").as_str(), sun_data);
    } else {
        cycle = parse_cycle_offline(format!("{working_directory}/wallpaper.config").as_str(),);
    }

    //-- loop ---
    loop {
        // if now online but wasn't online before recalculate timestamps with suntimes
        if !is_online && online::check(None).is_ok() {
            let sun_data = get_sun_data(additional_data.lat, additional_data.lng);
            cycle = parse_cycle_online(format!("{working_directory}/wallpaper.config").as_str(), sun_data);
        }

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