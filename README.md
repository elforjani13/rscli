# Random Data Sampling CLI Tool (rscli)
## Overview

`rscli` is a command-line tool written in Rust that samples lines from a delimited text file based on specified weights. It uses a heap-based approach to efficiently select random samples.

## Features

- **Weighted Sampling:** Selects lines from the input file based on provided weights.
- **Inclusion/Exclusion Filters:** Allows filtering lines based on specified IDs.
- **Customizable Input:** Supports specifying the input file, sample count, weight column, and more.
- **Logging:** Utilizes the `env_logger` crate for configurable logging.

## Usage

### Installation

Ensure you have Rust installed. You can then build and install the `rscli` executable using the following commands:

```bash
git clone https://github.com/yourusername/rscli.git
cd rscli
cargo build --release

rscli [OPTIONS]

OPTIONS:
    --file <file>                      Input tree file
    --sample_count <sample_count>      The number of samples to get
    --weights <weights>                The column with the weights
    --include <include>                Include these rows - names by Id column
    --exclude <exclude>                Exclude these rows - names by Id column
    --id_col <id_col>                  Id column - default is the first one

rscli [OPTIONS] --file <FILE> --sample-count <SAMPLE_COUNT>
```
### Example

```bash
# Sample 100 lines from input.txt with default settings
rscli --file input.txt --sample-count 100

# Sample 50 lines from input.txt with custom weight column and inclusion filter
rscli --file input.txt --sample-count 50 --weights my_weights --include id1,id2

# Sample 200 lines from data.csv excluding specific IDs
rscli --file data.csv --sample-count 200 --exclude id3,id4

```



