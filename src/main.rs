use std::{path::PathBuf, time::Instant};

use clap::Parser;

use rdupl::process_dir;

#[derive(Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    cache: Option<PathBuf>,

    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let t0 = Instant::now();

    let file_index = process_dir(args.path);

    let t1 = Instant::now();
    println!("{:.3}s", (t1 - t0).as_secs_f64());
    println!("files = {}", file_index.len());

    for (data, infos) in file_index.get_preview() {
        println!("\n{data}");
        for info in infos {
            println!("{info}");
        }
    }

    if let Some(cache_path) = args.cache
        && let Err(err) = file_index.dump(&cache_path)
    {
        eprintln!("Error dump to {}: {}", cache_path.display(), err);
    }
}
