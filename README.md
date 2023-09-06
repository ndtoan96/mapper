![build status](https://github.com/ndtoan96/mapper/actions/workflows/rust.yml/badge.svg)

# mapper
CLI tool to parse and extract symbols inormation from linker map file

# Usage
Download the tool [here](https://github.com/ndtoan96/mapper/releases)

```
Usage: mapper.exe [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   input map file
  [OUTPUT]  output file name (extension will be added according to selected format) [default: ./output]

Options:
  -f, --format <FORMAT>  output format [default: csv] [possible values: csv, json]
  -h, --help             Print help
  -V, --version          Print version
```
