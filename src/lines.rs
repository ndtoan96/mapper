use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many1;
use nom::sequence::*;
use nom::IResult;

use crate::types::*;
use crate::units::*;

pub fn empty_till_end_of_line(input: &str) -> IResult<&str, &str> {
    recognize(pair(space0, line_ending))(input)
}

pub fn assignment_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        space1,
        opt(tag("[")),
        address,
        opt(tag("]")),
        space1,
        assignment,
        line_ending,
    )))(input)
}

pub fn load_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("LOAD"), space1, path, empty_till_end_of_line)))(input)
}

pub fn output_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("OUTPUT"), not_line_ending, line_ending)))(input)
}

pub fn fill_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        space1,
        tag("*fill*"),
        space1,
        address,
        space1,
        hex_number,
        space0,
        digit0,
        empty_till_end_of_line,
    )))(input)
}

pub fn symbol_line(input: &str) -> IResult<&str, Symbol> {
    let (input, (_, address, _, sym, _)) =
        tuple((space1, address, space1, symbol, empty_till_end_of_line))(input)?;
    Ok((input, Symbol { name: sym, address }))
}

pub fn section_declaration(input: &str) -> IResult<&str, Section> {
    let (input, (sec_name, _, _, _, addr, _, size, _, _)) = tuple((
        section_name,
        space0,
        opt(preceded(
            tag("memory region -> "),
            alt((tag("*default*"), identifier)),
        )),
        multispace0,
        address,
        space1,
        hex_number,
        opt(tuple((space1, tag("load address"), space1, hex_number))),
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

pub fn section_rule_line(input: &str) -> IResult<&str, &str> {
    let (input, (_, rule, _, _)) = tuple((space1, section_rule, space0, line_ending))(input)?;
    Ok((input, rule))
}

pub fn file_section(input: &str) -> IResult<&str, FileSection> {
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

pub fn empty_section_line(input: &str) -> IResult<&str, &str> {
    recognize(pair(section_name, empty_till_end_of_line))(input)
}

pub fn function_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        space1,
        address,
        space1,
        many1(alt((
            tag(","),
            tag("("),
            tag(")"),
            tag("."),
            tag(" "),
            tag("_"),
            alphanumeric1,
        ))),
        empty_till_end_of_line,
    )))(input)
}

pub fn comment_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("/"),
        alphanumeric1,
        tag("/"),
        empty_till_end_of_line,
    )))(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fill_line() {
        assert!(fill_line(" *fill*         0x0000000063b75c1c        0x4 \n").is_ok());
        assert!(fill_line(" *fill*         0x0001054d        0x3 00\n").is_ok());
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
    fn test_load_line() {
        let input = "LOAD c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/32/crtbegin.o\r\n";
        let result: Result<(&str, &str), nom::Err<nom::error::Error<&str>>> = load_line(input);
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
        let input5 = " .text          0x00000000634519e0       0xe0 c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/libmingw32.a(lib32_libmingw32_a-atonexit.o)\n";
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

        assert_eq!(
            file_section(input5),
            Ok((
                "",
                FileSection {
                    section: ".text",
                    address: "0x00000000634519e0",
                    size: "0xe0",
                    file: "c:/toolbase/_ldata/mingw/comp_5.3.0_w64_2f/bin/../lib/gcc/x86_64-w64-mingw32/5.3.0/../../../../x86_64-w64-mingw32/lib/../lib32/libmingw32.a(lib32_libmingw32_a-atonexit.o)"
                }
            ))
        );
    }
}
