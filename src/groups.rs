use nom::branch::alt;
use nom::multi::many0;
use nom::sequence::*;
use nom::IResult;

use crate::lines::*;
use crate::types::*;

pub fn prefix_junk(input: &str) -> IResult<&str, &str> {
    let marker = "Linker script and memory map";
    match input.find(marker) {
        Some(index) => Ok((&input[(index + marker.len())..], marker)),
        None => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        ))),
    }
}

pub fn file_section_group(input: &str) -> IResult<&str, FileSectionGroup> {
    let (input, (file_section, symbols)) = tuple((file_section, many0(symbol_line)))(input)?;
    Ok((
        input,
        FileSectionGroup {
            file_section,
            symbols,
        },
    ))
}

pub fn section_group(input: &str) -> IResult<&str, SectionGroup> {
    fn assignment_line_wrapper(input: &str) -> IResult<&str, Option<FileSectionGroup>> {
        let (input, _) = assignment_line(input)?;
        Ok((input, None))
    }

    fn file_section_group_wrapper(input: &str) -> IResult<&str, Option<FileSectionGroup>> {
        let (input, output) = file_section_group(input)?;
        Ok((input, Some(output)))
    }

    fn section_rule_line_wrapper(input: &str) -> IResult<&str, Option<FileSectionGroup>> {
        let (input, _) = section_rule_line(input)?;
        Ok((input, None))
    }

    fn fill_line_wrapper(input: &str) -> IResult<&str, Option<FileSectionGroup>> {
        let (input, _) = fill_line(input)?;
        Ok((input, None))
    }

    fn empty_till_end_of_line_wrapper(input: &str) -> IResult<&str, Option<FileSectionGroup>> {
        let (input, _) = empty_till_end_of_line(input)?;
        Ok((input, None))
    }

    let (input, (section, outputs)) = pair(
        section_declaration,
        many0(alt((
            assignment_line_wrapper,
            file_section_group_wrapper,
            fill_line_wrapper,
            empty_till_end_of_line_wrapper,
            section_rule_line_wrapper,
        ))),
    )(input)?;

    let mut file_section_groups = Vec::new();
    for o in outputs {
        if let Some(group) = o {
            file_section_groups.push(group);
        }
    }

    Ok((
        input,
        SectionGroup {
            section,
            file_section_groups,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_section_group() {
        let input = r" .text          0x00000000634519e0       0xe0 c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/libmingw32.a(lib32_libmingw32_a-atonexit.o)
        0x00000000634519e0                mingw_onexit
        0x0000000063451aa0                atexit
";
        let result = file_section_group(input);
        dbg!(&result);
        assert!(result.is_ok());
    }
}
