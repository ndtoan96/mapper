use std::fs;
use std::path::Path;

use csv::Writer;
use nom::branch::alt;
use nom::multi::many0;
use nom::sequence::*;
use nom::IResult;

mod groups;
mod lines;
mod types;
mod units;

use groups::*;
use lines::*;
use types::*;

pub fn parse(input: &str) -> IResult<&str, Vec<SectionGroup>> {
    preceded(
        pair(
            prefix_junk,
            many0(alt((empty_till_end_of_line, assignment_line, load_line))),
        ),
        many0(section_group),
    )(input)
}

pub fn to_json(info: &Vec<SectionGroup>, path: &Path) -> anyhow::Result<()> {
    let content = serde_json::to_string_pretty(info)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn to_csv(info: &Vec<SectionGroup>, path: &Path) -> anyhow::Result<()> {
    let mut records = Vec::new();
    for section_group in info {
        for file_section_group in &section_group.file_section_groups {
            for symbol in &file_section_group.symbols {
                records.push(Record {
                    symbol: symbol.name,
                    address: symbol.address,
                    file: file_section_group.file_section.file,
                    old_section: file_section_group.file_section.section,
                    new_section: section_group.section.name,
                });
            }
        }
    }
    records.sort_unstable_by_key(|r| r.symbol);

    let mut writer = Writer::from_path(path)?;
    for record in records {
        writer.serialize(&record)?;
    }
    Ok(())
}
