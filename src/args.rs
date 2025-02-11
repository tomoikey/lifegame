use clap_derive::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "0.12")]
    ratio: f64,
    #[clap(short, long, default_value = "100")]
    millis_per_frame: u64,
}

impl Args {
    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    pub fn millis_per_frame(&self) -> u64 {
        self.millis_per_frame
    }
}
