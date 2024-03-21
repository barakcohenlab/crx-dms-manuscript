use clap::Parser;

use ::dms_tools::alignment;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
struct MainArgs {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Parser, Debug)]
enum Subcommand {
    /// Extract subsequences from PAF alignments
    Alignment(AlignmentArgs),
    /// Map coding variants to barcodes in PAF alignments
    MapCodingVariants(MapCodingVariantsArgs),
}

// Parse a tuple of (usize, usize) from a string of the form "first,second"
fn parse_tuple(s: &str) -> Result<(usize, usize), String> {
    let mut tokens = s.split(',');
    let first = tokens.next().ok_or_else(|| "missing first value".to_string())?;
    let second = tokens.next().ok_or_else(|| "missing second value".to_string())?;
    let first = first.parse::<usize>().map_err(|e| format!("invalid first value: {:?}", e))?;
    let second = second.parse::<usize>().map_err(|e| format!("invalid second value: {:?}", e))?;
    Ok((first, second))
}

#[derive(Parser, Debug)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
struct AlignmentArgs {
    /// A PAF alignment file produced by running minimap2 with the `--cs=long` option.
    #[clap(parse(from_os_str), value_hint=clap::ValueHint::FilePath)]
    source: std::path::PathBuf,

    /// The starting base (inclusive, zero-indexed) and ending base (exclusive, zero-indexed)
    /// of the desired feature in template space coordinates.
    #[clap(long, value_parser = parse_tuple)]
    bounds: (usize, usize),

    /// Only return sequences without indels (with length matching the reference sequence)
    #[clap(long)]
    exact_length: bool
}

#[derive(Parser, Debug)]
#[clap(setting = clap::AppSettings::DeriveDisplayOrder)]
struct MapCodingVariantsArgs {
    /// A PAF alignment file produced by running minimap2 with the `--cs=long` option.
    #[clap(parse(from_os_str), value_hint=clap::ValueHint::FilePath)]
    source: std::path::PathBuf,

    /// The starting base (inclusive, zero-indexed) and ending base (exclusive, zero-indexed)
    /// of the CDS in template space coordinates.
    #[clap(long, value_parser = parse_tuple)]
    cds: (usize, usize),

    /// The starting base (inclusive, zero-indexed) and ending base (exclusive, zero-indexed)
    /// of the barcode in template space coordinates.
    #[clap(long, value_parser = parse_tuple)]
    bc: (usize, usize),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_args: MainArgs = MainArgs::parse();

    match main_args.subcommand {
        Subcommand::Alignment(alignment_args) => alignment(alignment_args),
        Subcommand::MapCodingVariants(map_coding_variants_args) => map_coding_variants(map_coding_variants_args),
    }
}

fn alignment(arguments: AlignmentArgs) -> Result<(), Box<dyn std::error::Error>> {
    for maybe_record in alignment::RecordReader::read(&arguments.source)? {
        if let Ok(record) = maybe_record {
            if let Some(alignment) = record.alignment_subset(arguments.bounds.0, arguments.bounds.1) {
                println!("{} {}", alignment.length_relative_to_reference(), alignment.raw_length());
                if !arguments.exact_length || alignment.length_relative_to_reference() == alignment.raw_length() {
                    let (_, sequence) = alignment.make_sequences();
                    println!("{}", sequence);
                }
            }
        }
    }

    Ok(())
}

fn map_coding_variants(arguments: MapCodingVariantsArgs) -> Result<(), Box<dyn std::error::Error>> {
    for maybe_record in alignment::RecordReader::read(&arguments.source)? {
        if let Ok(record) = maybe_record {
            if let (Some(cds), Some(bc)) = (
                record.alignment_subset(arguments.cds.0, arguments.cds.1),
                record.alignment_subset(arguments.bc.0, arguments.bc.1)
            ) {
                let variants = cds.call_coding_variants();
            }
        }
    }

    Ok(())
}