#!/usr/bin/env python3

import sys
import os
import fphd

if "SLURM_CPUS_PER_TASK" in os.environ:
    os.environ["POLARS_MAX_THREADS"] = os.environ["SLURM_CPUS_PER_TASK"]
    fphd.set_available_threads(int(os.environ["SLURM_CPUS_PER_TASK"]))
import polars

if __name__ == "__main__":
    source_file_path = sys.argv[1]
    expected_barcodes_file = sys.argv[2]
    output_file_path = sys.argv[3]

    # Load BC-variant map (generated from long read sequencing data)
    barcode_variant_map = polars.read_csv(expected_barcodes_file, separator="\t").drop("read_count")

    # Read in extracted barcodes and count # of reads per BC1
    counts = polars.scan_csv(source_file_path, separator="\t").group_by(["BC1", "BC2"]).agg([
        polars.count().cast(polars.UInt64).alias("read_count"),
    ])
    
    # Generate list of barcodes that are not exact matches to the list of expected barcodes
    unexpected_barcodes = counts.filter(polars.col("BC1").is_in(barcode_variant_map["BC"]).not_()).collect()["BC1"].unique()

    # Initialize list to hold corrections
    raw_barcode_corrections = []

    # Compute Hamming distances for each unexpected barcode to all expected barcodes, return any with a distance less than threshold
    for unexpected_barcode, potential_corrections in zip(unexpected_barcodes, fphd.nearby_within_threshold(unexpected_barcodes, barcode_variant_map["BC"], 1, False)):
        if len(potential_corrections) == 1: # If only one BC matches within the distance threshold, use that BC
            raw_barcode_corrections.append((unexpected_barcode, potential_corrections[0][0], potential_corrections[0][1]))
        elif len(potential_corrections) > 1: # Otherwise, check if the multiple matches are all to the same variant
            potential_assigned_variant = None
            shortest_distance = potential_corrections[0][1]
            for potential_bc, distance in filter(lambda t: t[1] == shortest_distance, potential_corrections):
                variant_info = barcode_variant_map.filter(polars.col("BC") == potential_bc)
                if potential_assigned_variant is None:
                    potential_assigned_variant = variant_info["var_ref"].item() + str(variant_info["var_pos"].item()) + variant_info["var_alt"].item()
                elif potential_assigned_variant != (variant_info["var_ref"].item() + str(variant_info["var_pos"].item()) + variant_info["var_alt"].item()):
                    potential_assigned_variant = ""
            if potential_assigned_variant is not None and potential_assigned_variant != "":
                bc, distance = potential_bc, shortest_distance

    # Create table of corrections
    barcode_corrections = polars.DataFrame(raw_barcode_corrections, schema={"uncorrected_BC1": None, "corrected_BC1": None, "corrected_BC1_distance": polars.UInt64})

    # Join counts table with corrections table and correct BCs
    counts = counts.join(barcode_corrections.lazy(), how="left", left_on="BC1", right_on="uncorrected_BC1").with_columns([
        polars.when(polars.col("corrected_BC1").is_null().not_()).then(polars.col("BC1")).otherwise(None).alias("uncorrected_BC1"),
        polars.when(polars.col("corrected_BC1").is_null().not_()).then(polars.col("corrected_BC1")).otherwise(polars.col("BC1")).alias("BC1")
    ]).sort(["read_count"], descending=True).drop("corrected_BC1")

    # Join counts table with variant map, assigning based on corrected BC
    counts = counts.join(barcode_variant_map.lazy(), how="left", left_on="BC1", right_on="BC").collect()

    # Write out table of counted and assigned variants
    counts.write_parquet(output_file_path)
