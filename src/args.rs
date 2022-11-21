use clap::Parser;

/// create timed wallpapers
#[derive(Parser, Debug)]
#[command(author = "vequa", version = "0.2.4")]
pub struct Args {
   /// path to a folder containing the wallpaper.config and data.ini file
   #[arg(short, long, long_help = "path to a folder containing:\n\
                                    - wallpaper.config\n\
                                        file containing time-wallpaper relationship\n\
                                        !IMPORTANT! time shortcuts have to be followed with :[defaulttime]\n\
                                    - data.ini\n\
                                        file containing additional data for extra functionality"
        )]
   pub folder: String,

   /// number of seconds to wait between updates
   #[arg(short, long, default_value_t = 600)]
   pub delay: u64,
}