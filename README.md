# noteboook_converter

Converts Jupyter Notebooks into scripts.
Code will be separated into bash code, markdown and source code.

## Usage

Install via:  
```$ git clone https://github.com/lmeller-git/notebook_converter```

To convert a Notebook into a new target directory:
```$ cargo run --release -- --file <path to jupyter notebook> --target <path to target dir>```
