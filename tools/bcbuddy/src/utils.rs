pub mod fastq;

use phf::phf_map;

pub fn reverse_complement(sequence: &str) -> String {
    sequence.chars().rev().map(|n| COMPLEMENT.get(&n).expect("attempted to reverse-complement non-ATCGN nucleotide")).collect()
}

static COMPLEMENT: phf::Map<char, char> = phf_map! {
    'A' => 'T',
    'C' => 'G',
    'T' => 'A',
    'G' => 'C',
    'N' => 'N',
};
