use std::fs::File;

use bw_img::{BWByteData, IterDirection, IterOutput};
use clap::Parser;
use flate2::{write::ZlibEncoder, Compression};

#[cfg(feature = "image")]
mod img;
#[cfg(feature = "video")]
mod vid;

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(clap::Args)]
#[group(id = "input-type", required = true, multiple = false)]
struct ConvertType {
    /// read as video file
    #[cfg(feature = "video")]
    #[arg(short, long)]
    video: bool,
    /// read as image file
    #[cfg(feature = "image")]
    #[arg(short, long)]
    image: bool,
}

#[derive(clap::Args)]
struct ConvertArgs {
    #[clap(flatten)]
    typ: ConvertType,
    #[arg(required = true, index = 1)]
    path: String,
    /// Output file
    #[arg(short, long)]
    output: Option<String>,
    /// Output height
    #[arg(long)]
    height: Option<u32>,
    /// Output width
    #[arg(long)]
    width: Option<u32>,
}

#[derive(clap::Args)]
struct ShowArgs {
    /// zipped bw imgs file path
    #[arg(required = true)]
    path: String,
    /// Index of the image to show
    #[arg(short, long, default_value_t = 0)]
    index: usize,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Convert video or image to zipped bw imgs
    Convert(ConvertArgs),
    /// Show bw image from zipped bw imgs
    Show(ShowArgs),
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.commands {
        Commands::Convert(args) => convert(args),
        Commands::Show(args) => show(args),
    }
}

fn convert(args: ConvertArgs) -> anyhow::Result<()> {
    #[cfg(feature = "video")]
    if args.typ.video {
        let mut file = File::create(args.output.unwrap_or("output.imgs".into()))?;
        let mut encoder = ZlibEncoder::new(&mut file, Compression::best());
        println!("Processing video...");
        match vid::transform_vid(&args.path, args.width, args.height, &mut encoder) {
            Ok((p, s)) => {
                encoder.finish()?;
                println!("processed {} frames, skipped {} frames", p, s);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error occurred while processing video.");
                Err(e)
            }
        }
    } else {
        unimplemented!()
    }
}

fn show(args: ShowArgs) -> anyhow::Result<()> {
    let mut file = std::fs::File::open(&args.path)?;
    println!("Decompressing {}...", args.path);

    let imgs = bw_img::file::zip::decompress_imgs(&mut file)?;

    fn write_pixel(is_white: bool) {
        if is_white {
            print!("██");
        } else {
            print!("  ");
        }
    }

    match imgs.get(args.index) {
        Some(img) => {
            for ele in img.iterator(IterDirection::Horizontal) {
                match ele {
                    IterOutput::Byte { byte, len } => {
                        for ele in byte.bw_byte_iter(len) {
                            write_pixel(ele);
                        }
                    }
                    IterOutput::NewLine => println!(),
                }
            }
        }
        None => {
            eprintln!("Invalid image index!");
        }
    }

    Ok(())
}
