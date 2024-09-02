#!/bin/bash

# variables: sample_rep sample_bin raw_read1_files raw_read2_files

sample_name=CTRA-${sample_rep}${sample_bin}

input_dir=$(mktemp -d)
working_dir=$(mktemp -d)
output_dir=CleanedReads

mkdir -p ${output_dir}

IFS=',' read -r -a read1_files <<< "$raw_read1_files"

for index in "${!read1_files[@]}"
do
    cat ${input_dir}/${read1_files[index]} >> ${working_dir}/${sample_name}_R1.fastq.gz
done

fastp --thread 2 --dont_eval_duplication --html ${output_dir}/${sample_name}.fastp.report.html --json ${output_dir}/${sample_name}.fastp.report.json --report_title ${sample_name} --in1 ${working_dir}/${sample_name}_R1.fastq.gz --out1 ${output_dir}/${sample_name}.fastq.gz || exit 1
