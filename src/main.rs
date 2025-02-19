use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    file: PathBuf,
    #[arg(short, long, default_value = ".")]
    target: PathBuf,
}

type Result<T> = std::result::Result<T, ConverterError>;

#[derive(Error, Debug)]
enum ConverterError {
    #[error("could not open file, {0:#?}")]
    IOError(#[from] std::io::Error),
    #[error("could not parse file, {0:#?}")]
    ParseError(#[from] serde_json::Error),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct NoteBook {
    #[serde(rename = "cells")]
    contents: Vec<NamedContent>,
    metadata: Metadata,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Metadata {
    content: Option<serde_json::Value>,
    language_info: LanguageInfo,
}

#[derive(Default, Deserialize, Serialize, Debug)]
struct LanguageInfo {
    file_extension: String,
    name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct NamedContent {
    cell_type: String,
    #[serde(rename = "source", default)]
    content: Vec<String>,
}

fn parse_file(f: &Path) -> Result<NoteBook> {
    let f = fs::File::open(f)?;
    Ok(serde_json::from_reader(f)?)
}

fn is_printout(line: &str) -> bool {
    let line = line.trim().trim_start();
    if line.is_empty() {
        return false;
    }
    for c in line.chars() {
        if !c.is_alphabetic() {
            return false;
        }
    }
    true
}

fn is_shell_code_cell(block: &[String]) -> bool {
    block
        .first()
        .unwrap_or(&"".to_string())
        .trim()
        .starts_with("%%")
}

fn is_shell_code(line: &str) -> bool {
    line.trim().starts_with('!') || line.trim().starts_with('%')
}

fn write_content(book: &NoteBook, target: &Path) -> Result<()> {
    if !target.is_dir() {
        fs::create_dir(target)?;
    } else {
        fs::File::create(target.join(format!(
            "content{}",
            book.metadata.language_info.file_extension
        )))?;
        fs::File::create(target.join("content.md"))?;
        fs::File::create(target.join("unknown.txt"))?;
        fs::File::create(target.join("content.sh"))?;
    }
    println!("created dir {}", target.display());
    for c in &book.contents {
        match c.cell_type.as_ref() {
            "code" => {
                let (mut handle, is_shell) = if is_shell_code_cell(&c.content) {
                    (
                        fs::File::options()
                            .append(true)
                            .create(true)
                            .open(target.join("content.sh"))?,
                        true,
                    )
                } else {
                    (
                        fs::File::options()
                            .append(true)
                            .create(true)
                            .open(target.join(format!(
                                "content{}",
                                book.metadata.language_info.file_extension
                            )))?,
                        false,
                    )
                };

                for line in &c.content[if is_shell { 1 } else { 0 }..] {
                    if is_shell_code(line) {
                        let mut shell_handle = fs::File::options()
                            .append(true)
                            .create(true)
                            .open(target.join("content.sh"))?;
                        let line = line.trim_start_matches('!').trim_start_matches('!');
                        shell_handle.write_all(line.as_bytes())?;
                        continue;
                    }
                    if book.metadata.language_info.name == "python"
                        && is_printout(line)
                        && !is_shell
                    {
                        handle.write_all(format!("print(\"{0} = \", {0})\n", line).as_bytes())?;
                    } else {
                        handle.write_all(line.as_bytes())?;
                        handle.write_all(b"\n")?;
                    }
                }
            }
            "markdown" => {
                let mut handle = fs::File::options()
                    .append(true)
                    .create(true)
                    .open(target.join("content.md"))?;

                for line in &c.content {
                    handle.write_all(line.as_bytes())?;
                    handle.write_all(b"  \n")?;
                }
            }
            _ => {
                let mut handle = fs::File::options()
                    .append(true)
                    .create(true)
                    .open(target.join("unknown.txt"))?;

                for line in &c.content {
                    handle.write_all(line.as_bytes())?;
                    handle.write_all(b"\n")?;
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    let content = match parse_file(&args.file) {
        Ok(c) => c,
        Err(_e) => {
            println!("err");
            return;
        }
    };
    // println!("{:#?}", content);
    println!("content parsed");
    println!("writing content into {} ...", args.target.display());
    match write_content(&content, &args.target) {
        Ok(()) => println!("content written to files"),
        Err(e) => println!("could not write content, {:#?}", e),
    }
}
