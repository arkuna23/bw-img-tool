use clap::Parser;
use crossterm::{style, ExecutableCommand};

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
        println!("Converting video to zipped bw imgs...");
        match vid::transform_vid(&args.path, args.width, args.height) {
            Ok(frames) => {
                let output = args.output.unwrap_or("output.imgs".into());
                let mut file = std::fs::File::create(&output).unwrap();
                bw_img::file::zip::compress_imgs(&frames, &mut file).unwrap();
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

    fn write_pixel(stdout: &mut std::io::Stdout, is_white: bool) -> anyhow::Result<()> {
        if is_white {
            stdout.execute(style::SetBackgroundColor(style::Color::White))?;
        } else {
            stdout.execute(style::SetForegroundColor(style::Color::Black))?;
        }

        print!("  ");
        Ok(())
    }

    match imgs.get(args.index) {
        Some(img) => {
            let mut stdout = std::io::stdout();

            let len = img.config.width;
            let mut curr_width = 0;
            for byt in img.data.iter() {
                for i in 0..8 {
                    let is_white = (byt >> (7 - i)) == 0;
                    write_pixel(&mut stdout, is_white)?;
                    curr_width += 1;

                    if curr_width == len {
                        stdout.execute(style::ResetColor)?;
                        println!();
                        curr_width = 0;
                    }
                }
            }
        }
        None => {
            eprintln!("Invalid image index!");
        }
    }

    Ok(())
}
