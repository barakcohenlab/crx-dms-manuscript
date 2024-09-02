#!/bin/bash

# variables: sample_rep sample_bin raw_read1_files raw_read2_files

sample_name=CTRA-${sample_rep}${sample_bin}

input_dir=CleanedReads
output_dir=Barcodes

mkdir -p ${output_dir}

zcat ${input_dir}/${sample_name}.fastq.gz | bcbuddy --source /dev/stdin --regex '(?P<BC1>[ATCG]{9}CA[ATCG]{9})AACTCTTACTGCCCAGTCCC(?P<BC2>[ATCG]{8}TG[ATCG]{8}CA[ATCG]{8})' --output ${output_dir}/${sample_name}.extracted_barcodes.tsv  --run-stats ${output_dir}/${sample_name}.stats.json
