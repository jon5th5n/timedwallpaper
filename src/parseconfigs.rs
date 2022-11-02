use crate::help::HELP;

use crate::sundata::get_sun_data_key_from_shortcut;
use std::{fs, collections::HashMap};
use chrono::{prelude::*, Duration};
use ini::Ini;


fn panic_help(s: &str) -> ! {
    println!("\n{s}");
    println!("{HELP}");
    panic!("{s}");
}


pub fn parse_cycle_online(filepath: &str, sun_data: HashMap<String, NaiveTime>) -> Vec<(NaiveTime, String)> {
    let config = {
        let this = fs::read_to_string(filepath);
        match this {
            Ok(t) => t,
            Err(_e) => panic_help("directory doesn't contain a file named wallpaper.config"),
        }
    }.replace(" ", "");

    // whole config => single entries
    let config: Vec<&str> = config.split('\n').collect();

    // entry => timestamp / vec of paths
    let mut config_timestamped: Vec<(NaiveTime, Vec<&str>)> = Vec::new();
    for tp in config {
        // entry => timestr or shortcut / str of paths
        let time_paths: Vec<&str> = tp.split('=').collect();

        let time: NaiveTime;
        // if shortcut
        if time_paths[0].starts_with("#") {
            // timestr or SHORTCUT => shortcut / defaulttimestr
            let time_str: Vec<&str> = time_paths[0].split(':').collect();
            let shortcut = time_str[0].get(1..).unwrap();
            time = sun_data.get(get_sun_data_key_from_shortcut(shortcut)).unwrap().clone();
        } else {
            // TIMESTR or shortcut => hour / minute
            let time_str: Vec<&str> = time_paths[0].split('_').collect();
            time = NaiveTime::from_hms(time_str[0].parse().unwrap(), time_str[1].parse().unwrap(), 0);
        }

        // str of paths => vec of paths
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

pub fn parse_cycle_offline(filepath: &str) -> Vec<(NaiveTime, String)> {
    let config = {
        let this = fs::read_to_string(filepath);
        match this {
            Ok(t) => t,
            Err(_e) => panic_help("directory doesn't contain a file named wallpaper.config"),
        }
    }.replace(" ", "");

    // whole config => single entries
    let config: Vec<&str> = config.split('\n').collect();

    // entry => timestamp / vec of paths
    let mut config_timestamped: Vec<(NaiveTime, Vec<&str>)> = Vec::new();
    for tp in config {
        // entry => timestr or shortcut / str of paths
        let time_paths: Vec<&str> = tp.split('=').collect();

        let time: NaiveTime;

        // timestr or shortcut-defaulttimestr => hour / minute
        let time_str: Vec<&str> = if time_paths[0].starts_with("#") {
            time_paths[0].split(':').collect::<Vec<&str>>()[1].split('_').collect()
        } else {
            time_paths[0].split('_').collect()
        };

        time = NaiveTime::from_hms(time_str[0].parse().unwrap(), time_str[1].parse().unwrap(), 0);

        // str of paths => vec of paths
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


pub struct AdditionalData {
    pub lat: f32,
    pub lng: f32,
}

pub fn parse_data(filepath: &str) -> AdditionalData {
    let config_data = {
        let this = Ini::load_from_file(filepath);
        match this {
            Ok(t) => t,
            Err(_e) => panic_help("directory doesn't contain a file named data.ini"),
        }
    };

    let config_general = {
        let this = config_data.section(Some("General"));
        match this {
            Some(val) => val,
            None => panic_help("data.ini file doesn't include General section"),
        }
    };

    let lat = {
        let this = config_general.get("latitude");
        match this {
            Some(val) => val,
            None => panic_help("data.ini file doesn't include latitude inside General section"),
        }
    }.parse::<f32>().unwrap();
    let lng = {
        let this = config_general.get("longitude");
        match this {
            Some(val) => val,
            None => panic_help("data.ini file doesn't include longitude inside General section"),
        }
    }.parse::<f32>().unwrap();

    AdditionalData { lat, lng }
}