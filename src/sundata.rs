use std::collections::HashMap;
use chrono::{prelude::*, Duration};
use serde_json::{Map, Value};

const SUN_DATA: &'static [&'static str] = &["civil_twilight_begin", "sunrise", "solar_noon", "sunset", "civil_twilight_end", "solar_midnight"];
pub fn get_sun_data_key_from_shortcut(sc: &str) -> &str {
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

#[derive(serde::Serialize, serde::Deserialize)]
struct ApiData {
    results: Map<String, Value>,
    status: String,
}

pub fn get_sun_data(lat: f32, lng: f32) -> HashMap<String, NaiveTime> {

    let res_string = {
        let this = reqwest::blocking::get(format!("https://api.sunrise-sunset.org/json?lat={lat}&lng={lng}"));
        match this {
            Ok(t) => t,
            Err(_e) => panic!("unable to get sundata from api"),
        }
    }.text().unwrap();
    let res: ApiData = serde_json::from_str(res_string.as_str()).unwrap();
    let res = res.results;

    // Value(String) => NaiveTime
    let mut results: HashMap<String, NaiveTime> = HashMap::new();
    for (k, v) in res {
        if !SUN_DATA.contains(&k.as_str()) { continue };

        let s = v.as_str().unwrap();

        // timestr + AM or PM => timestr / AM or PM
        let tmp: Vec<&str> = s.split(' ').collect();
        let (time, pm) = (tmp[0], tmp[1]);
        let pm = if pm == "PM" { true } else { false };

        // timestr => hour / minute / second
        let hms: Vec<&str> = time.split(':').collect();
        let (h, m, s) = (hms[0].parse().unwrap(), hms[1].parse().unwrap(), hms[2].parse().unwrap());
        let mut time = NaiveTime::from_hms(h, m, s);
        if pm {
            time += Duration::hours(12);
        }

        let timezone_offset = Local::now().offset().to_string();
        // offset => offset-hour / offset-minute
        let timezone_offset: Vec<&str> = timezone_offset.get(1..).unwrap().split(':').collect();
        time += Duration::minutes(60 * timezone_offset[0].parse::<i64>().unwrap() + timezone_offset[1].parse::<i64>().unwrap());

        results.insert(k, time);
    }

    // add solar_midnight (solar_noon + 12h)
    results.insert("solar_midnight".to_string(), results.get("solar_noon").unwrap().clone() + Duration::hours(12));

    results
}