#!/bin/bash

# variables: sample_rep sample_bin raw_read1_files raw_read2_files

sample_name=CTRA-${sample_rep}${sample_bin}

data_dir=Barcodes

./count_bcs.py ${data_dir}/${sample_name}.extracted_barcodes.tsv external_data_sources/barcode_to_variant_map.tsv ${data_dir}/${sample_name}.extracted_barcodes.counts.parquet
