use std::fs;

use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::*;
use nom::IResult;

fn prefix_junk(input: &str) -> IResult<&str, &str> {
    let marker = "Linker script and memory map";
    match input.find(marker) {
        Some(index) => Ok((&input[(index + marker.len())..], marker)),
        None => Err(nom::Err::Failure(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Eof,
        ))),
    }
}

fn empty_till_end_of_line(input: &str) -> IResult<&str, &str> {
    recognize(pair(space0, line_ending))(input)
}

fn address(input: &str) -> IResult<&str, &str> {
    hex_number(input)
}

fn hex_number(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        tag::<_, _, nom::error::Error<&str>>("0x"),
        nom::bytes::complete::take_while1(|c: char| c.is_digit(16)),
    ))(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("_"), tag("."), alphanumeric1))))(input)
}

fn symbol(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("_"), tag("@"), tag(","), tag("("), tag(")"), tag(" "), alphanumeric1))))(input)
}

fn assignment(input: &str) -> IResult<&str, &str> {
    recognize(tuple((identifier, space0, tag("="), not_line_ending)))(input)
}

fn assignment_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((space1, address, space1, assignment, line_ending)))(input)
}

fn path_delimiter(input: &str) -> IResult<&str, &str> {
    alt((tag("/"), tag("\\")))(input)
}

fn root(input: &str) -> IResult<&str, &str> {
    let windows_root = recognize(tuple((alpha1, tag(":"), path_delimiter)));
    let linux_root = path_delimiter;
    alt((windows_root, linux_root))(input)
}

fn directory(input: &str) -> IResult<&str, &str> {
    recognize(pair(path_name, path_delimiter))(input)
}

fn path_name(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((alphanumeric1, tag("."), tag("-"), tag("_")))))(input)
}

fn file_name(input: &str) -> IResult<&str, &str> {
    path_name(input)
}

fn path(input: &str) -> IResult<&str, &str> {
    let true_file_name = recognize(tuple((tag("("), identifier, tag(")"))));
    recognize(tuple((
        opt(root),
        many0(directory),
        file_name,
        opt(true_file_name),
    )))(input)
}

fn load_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("LOAD"), space1, path, empty_till_end_of_line)))(input)
}

fn section_name(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("."), tag("$"), alphanumeric1, tag("_")))))(input)
}

fn section_rule(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((
        tag("."),
        alphanumeric1,
        tag("("),
        tag(")"),
        tag("*"),
        tag("$"),
        tag("_"),
    ))))(input)
}

#[derive(Debug, PartialEq)]
struct Section<'a> {
    name: &'a str,
    address: &'a str,
    size: &'a str,
}

fn section_declaration(input: &str) -> IResult<&str, Section> {
    let (input, (sec_name, _, _, addr, _, size, _)) = tuple((
        section_name,
        opt(preceded(
            tag(" memory region -> "),
            alt((tag("*default*"), alphanumeric1)),
        )),
        multispace1,
        address,
        space1,
        hex_number,
        empty_till_end_of_line,
    ))(input)?;
    Ok((
        input,
        Section {
            name: sec_name,
            address: addr,
            size,
        },
    ))
}

fn section_rule_line(input: &str) -> IResult<&str, &str> {
    let (input, (_, rule, _, _)) = tuple((space1, section_rule, space0, line_ending))(input)?;
    Ok((input, rule))
}

#[derive(Debug, PartialEq)]
struct FileSection<'a> {
    section: &'a str,
    file: &'a str,
    address: &'a str,
    size: &'a str,
}

fn file_section(input: &str) -> IResult<&str, FileSection> {
    let (input, (_, sec_name, _, addr, _, size, _, file, _)) = tuple((
        space1,
        section_name,
        multispace1,
        address,
        space1,
        hex_number,
        space1,
        path,
        empty_till_end_of_line,
    ))(input)?;
    Ok((
        input,
        FileSection {
            section: sec_name,
            file,
            address: addr,
            size,
        },
    ))
}

#[derive(Debug, PartialEq)]
struct Symbol<'a> {
    name: &'a str,
    address: &'a str,
}

fn symbol_line(input: &str) -> IResult<&str, Symbol> {
    let (input, (_, address, _, sym, _)) =
        tuple((space1, address, space1, symbol, empty_till_end_of_line))(input)?;
    Ok((input, Symbol { name: sym, address }))
}

#[derive(Debug, PartialEq)]
struct FileSectionGroup<'a> {
    file_section: FileSection<'a>,
    symbols: Vec<Symbol<'a>>,
}

fn fill_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        space1,
        tag("*fill*"),
        space1,
        address,
        space1,
        hex_number,
        empty_till_end_of_line,
    )))(input)
}

fn file_section_group(input: &str) -> IResult<&str, FileSectionGroup> {
    let (input, (file_section, symbols, _)) =
        tuple((file_section, many0(symbol_line), opt(fill_line)))(input)?;
    Ok((
        input,
        FileSectionGroup {
            file_section,
            symbols,
        },
    ))
}

#[derive(Debug, PartialEq)]
struct SectionGroup<'a> {
    section: Section<'a>,
    file_section_groups: Vec<FileSectionGroup<'a>>,
}

fn section_group(input: &str) -> IResult<&str, SectionGroup> {
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

    fn empty_till_end_of_line_wrapper(input: &str) -> IResult<&str, Option<FileSectionGroup>> {
        let (input, _) = empty_till_end_of_line(input)?;
        Ok((input, None))
    }

    let (input, (section, outputs)) = pair(
        section_declaration,
        many0(alt((
            assignment_line_wrapper,
            file_section_group_wrapper,
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

fn parse(input: &str) -> IResult<&str, Vec<SectionGroup>> {
    preceded(
        pair(prefix_junk, many0(alt((empty_till_end_of_line, assignment_line, load_line)))),
        many0(section_group),
    )(input)
}

fn main() {
    // let input = fs::read_to_string("relocatable.map").unwrap();
    let input = fs::read_to_string("vecuTasks.map").unwrap();
    let (input, output) = parse(&input).unwrap();
    println!("{}", &input[..400]);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_identifier() {
        let input = "__image_base__";
        let result = identifier(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assignment() {
        let input = "__image_base__ = 0x632c0000";
        let result = assignment(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assignment_line() {
        let input = r"         0x00000000632c0000                __image_base__ = 0x632c0000
        ";
        let result = assignment_line(input);
        assert!(result.is_ok());
        println!("{:?}", result.unwrap().0);
    }

    #[test]
    fn test_path() {
        let input = "c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/32/crtbegin.o";
        let result = path(input);
        assert!(result.is_ok());

        let input = "_gen/swb/filegroup/linker/libs/_prj_link_archive.a(_merged_dat.o)";
        let result = path(input);
        assert!(result.is_ok());

        assert!(path("c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/libmingw32.a(lib32_libmingw32_a-atonexit.o)").is_ok());
    }

    #[test]
    fn test_load_line() {
        let input = "LOAD c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/32/crtbegin.o\r\n";
        let result: Result<(&str, &str), nom::Err<nom::error::Error<&str>>> = load_line(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_section_name() {
        assert!(section_name(".text").is_ok());
        assert!(section_name(".text.bsw").is_ok());
        assert!(section_name(".data.a4").is_ok());
        assert!(section_name(".bss.a4..someVAR").is_ok());
    }

    #[test]
    fn test_section_rule() {
        assert!(section_rule("*(.data$Gpf2Exh_FacEnthpyCorrnX_A)").is_ok());
        assert!(section_rule("*(.init)").is_ok());
    }

    #[test]
    fn test_section_declaration() {
        let input1 = r".data.SWRESET.PRAM3 memory region -> *default*
        0x000e0000        0x0
";
        let input2 = r".flashConfigData_empty memory region -> flashConfigArea
        0x800a8e34     0x11cc
";
        let input3 = r".text           0x00000000632c1000   0x762200
";
        assert_eq!(
            section_declaration(input1),
            Ok((
                "",
                Section {
                    name: ".data.SWRESET.PRAM3",
                    address: "0x000e0000",
                    size: "0x0"
                }
            ))
        );
        assert_eq!(
            section_declaration(input2),
            Ok((
                "",
                Section {
                    name: ".flashConfigData_empty",
                    address: "0x800a8e34",
                    size: "0x11cc"
                }
            ))
        );
        assert_eq!(
            section_declaration(input3),
            Ok((
                "",
                Section {
                    name: ".text",
                    address: "0x00000000632c1000",
                    size: "0x762200"
                }
            ))
        );
    }

    #[test]
    fn test_file_section() {
        let input1 = " .text          0x00000000632c1000      0x450 c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/dllcrt2.o\r\n";
        let input2 = r" .bss.a1..DFES_stOutstate
        0x001b7d5b        0x1 _gen/swb/filegroup/linker/libs/_prj_link_archive.a(dfes_outstate.o)
";
        let input3 = r" .zbss.SWRESET.ZRAM3_mcop
        0x40000090      0x170 _gen/swb/module/build/reloc_vared.elf
";
        let input4 = r" .idata$5       0x000000006711f38c        0x4 c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/libmsvcrt.a(dqgfs01158.o)
";
        assert_eq!(file_section(input1), Ok(("", FileSection {
            section: ".text",
            address: "0x00000000632c1000",
            size: "0x450",
            file: "c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/dllcrt2.o"
        })));

        assert_eq!(
            file_section(input2),
            Ok((
                "",
                FileSection {
                    section: ".bss.a1..DFES_stOutstate",
                    address: "0x001b7d5b",
                    size: "0x1",
                    file: "_gen/swb/filegroup/linker/libs/_prj_link_archive.a(dfes_outstate.o)"
                }
            ))
        );

        assert_eq!(
            file_section(input3),
            Ok((
                "",
                FileSection {
                    section: ".zbss.SWRESET.ZRAM3_mcop",
                    address: "0x40000090",
                    size: "0x170",
                    file: "_gen/swb/module/build/reloc_vared.elf"
                }
            ))
        );

        assert_eq!(
            file_section(input4),
            Ok((
                "",
                FileSection {
                    section: ".idata$5",
                    address: "0x000000006711f38c",
                    size: "0x4",
                    file: "c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/libmsvcrt.a(dqgfs01158.o)"
                }
            ))
        );
    }

    #[test]
    fn test_symbol_line() {
        let input1 = "                0x6000016c                B_sldmnws\r\n";
        let input2 = "                0x000000006711f270                _imp__StackWalk@36\r\n";
        assert_eq!(
            symbol_line(input1),
            Ok((
                "",
                Symbol {
                    name: "B_sldmnws",
                    address: "0x6000016c"
                }
            ))
        );
        assert_eq!(
            symbol_line(input2),
            Ok((
                "",
                Symbol {
                    name: "_imp__StackWalk@36",
                    address: "0x000000006711f270"
                }
            ))
        );
    }

    #[test]
    fn test_fill_line() {
        assert!(fill_line(" *fill*         0x0000000063b75c1c        0x4 \n").is_ok());
    }
}
