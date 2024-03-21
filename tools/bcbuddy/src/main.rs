use std::fs;
use std::io::prelude::*;

use clap::{Parser, value_parser, CommandFactory};
use itertools::Itertools;

mod barcodes;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Arguments {
    /// A FASTQ file containing amplicon reads from which to extract barcodes
    #[arg(short, long, required = true, value_parser = value_parser!(std::path::PathBuf), value_hint = clap::ValueHint::FilePath)]
    source: Vec<std::path::PathBuf>,

    /// A path to a file in which extracted barcodes should be written.
    #[arg(short, long, value_parser = value_parser!(std::path::PathBuf), value_hint = clap::ValueHint::FilePath)]
    output: std::path::PathBuf,

    /// The regex string matching the barcode(s). Should contain one or more
    /// capture groups
    #[arg(short, long, required = true)]
    regex: Vec<String>,

    /// If set, the returned capture groups will be reverse-complemented. This occurs *after* regex
    /// matching.
    #[arg(short, long)]
    reverse_complement_output: bool,

    /// If set, FASTQ read IDs will be printed as a column in the output
    #[arg(short('i'), long)]
    output_read_ids: bool,

    /// A path to a file in which run statistics should be written (JSON format)
    #[arg(short('t'), long, value_parser = value_parser!(std::path::PathBuf), value_hint = clap::ValueHint::FilePath)]
    run_stats: Option<std::path::PathBuf>,

    /*
    /// A path to a file in which reads that do not match the regex should be written (FASTQ
    /// format)
    #[arg(short, long, value_parser = value_parser!(std::path::PathBuf), value_name = "PATH", value_hint = clap::ValueHint::FilePath)]
    unmatched_reads: Vec<std::path::PathBuf>,
    */
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Arguments::parse();
    
    if arguments.source.len() != arguments.regex.len() {
        Arguments::command().error(clap::error::ErrorKind::WrongNumberOfValues, format!("The number of regexes ({}) must match the number of sources ({})", arguments.regex.len(), arguments.source.len())).exit();
    }

    let source_files: Vec<fs::File> = match arguments.source.into_iter().map(fs::File::open).collect() {
        Ok(source_files) => source_files,
        Err(error) => { return Err(Box::new(error)); }
    };

    let mut out = fs::File::create(arguments.output)?;

    let mut stats_out = arguments.run_stats.map(|path| fs::File::create(path));
    if let Some(Err(error)) = stats_out {
        return Err(Box::new(error));
    }

    /*
    let mut unmatched_outs = arguments.unmatched_reads.map(|path| fs::File::create(path));
    if let Some(Err(error)) = unmatched_out {
        return Err(Box::new(error));
    }
    */

    let extractors: Vec<barcodes::BarcodeExtractor> = match arguments.regex.iter().map(String::as_str).map(barcodes::BarcodeExtractor::new).collect() {
        Ok(extractors) => extractors,
        Err(error) => { return Err(Box::new(error)); }
    };

    if arguments.output_read_ids {
        write!(out, "{}\t", (1..=source_files.len()).map(|i| format!("read{}_id", i)).join("\t"))?;
    }

    writeln!(out, "{}", extractors.iter().map(|extractor| {
        extractor.capture_group_names().into_iter().join("\t")
    }).join("\t"))?;

    let mut reads_iterators: Vec<utils::fastq::FASTQReader<_>> = source_files.into_iter().map(utils::fastq::FASTQReader::read_fastq).collect();

    let mut num_reads: usize = 0;
    let mut num_matching_reads: usize = 0;
    loop {
        let all_records: Vec<utils::fastq::FASTQRecord> = match reads_iterators.iter_mut().map(|i| i.next()).collect() {
            Some(Ok(records)) => records,
            Some(Err(error)) => { return Err(Box::new(error)); },
            None => {
                break;
            }
        };
        num_reads += 1;

        let all_captures: Vec<Vec<&str>> = match all_records.iter().zip(extractors.iter()).map(|(record, extractor)| { extractor.extract(&record.sequence) }).collect() {
            Some(captures) => captures,
            None => {
                //TODO: Write record to unmatched_out
                continue;
            }
        };
        num_matching_reads += 1;

        if arguments.output_read_ids {
            write!(out, "{}\t", all_records.iter().map(|r| &r.identifier).join("\t"))?;
        }
        
        writeln!(out, "{}", all_captures.iter().map(|captures| {
            if arguments.reverse_complement_output {
                captures.into_iter().map(|sequence| utils::reverse_complement(sequence)).join("\t")
            } else {
                captures.into_iter().join("\t")
            }
        }).join("\t"))?;
    }

    if let Some(Ok(ref mut stats_out)) = stats_out {
        writeln!(stats_out, "{{")?;
        writeln!(stats_out, "\t\"total_reads\": {},", num_reads)?;
        writeln!(stats_out, "\t\"reads_with_BC\": {}", num_matching_reads)?;
        write!(stats_out, "}}")?;
    }

    Ok(())
}
