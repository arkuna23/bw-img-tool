use clap::Parser;

#[cfg(feature = "image")]
mod img;
#[cfg(feature = "video")]
mod vid;
mod bw;

#[derive(clap::Parser)]
struct Args {
    /// Video file to process
    #[cfg(feature = "video")]
    #[arg(short, long)]
    video: Option<String>,
    /// Image file to process
    #[cfg(feature = "image")]
    #[arg(short, long)]
    image: Option<String>,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Output height
    #[arg(short, long)]
    height: Option<u32>,
    /// Output width
    #[arg(short, long)]
    width: Option<u32>,
}

fn main() {
    let args = Args::parse();
}
