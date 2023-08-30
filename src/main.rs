use std::fs;

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

fn parse(input: &str) -> IResult<&str, Vec<SectionGroup>> {
    preceded(
        pair(
            prefix_junk,
            many0(alt((empty_till_end_of_line, assignment_line, load_line))),
        ),
        many0(section_group),
    )(input)
}

fn main() {
    let input = fs::read_to_string("relocatable.map").unwrap();
    // let input = fs::read_to_string("vecuTasks.map").unwrap();
    let (input, _) = parse(&input).unwrap();
    println!("{}", &input[..400]);
}
