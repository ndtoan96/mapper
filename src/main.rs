use clap::{Parser, ValueEnum};
use mapper::{parse, to_csv, to_json};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Csv,
    Json,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, value_enum, default_value = "csv", help = "output format")]
    format: Format,
    #[arg(help = "input map file")]
    input: PathBuf,
    #[arg(
        default_value = "./output",
        help = "output file name (extension will be added according to selected format)"
    )]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input = fs::read_to_string(args.input)?;
    let (_input, output) = parse(&input).unwrap();
    match args.format {
        Format::Csv => to_csv(&output, &args.output.with_extension("csv"))?,
        Format::Json => to_json(&output, &args.output.with_extension("json"))?,
    }
    Ok(())
}
