use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::multi::many1;
use nom::sequence::*;
use nom::IResult;

pub fn address(input: &str) -> IResult<&str, &str> {
    hex_number(input)
}

pub fn hex_number(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        tag::<_, _, nom::error::Error<&str>>("0x"),
        nom::bytes::complete::take_while1(|c: char| c.is_ascii_hexdigit()),
    ))(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("_"), tag("."), alphanumeric1))))(input)
}

pub fn symbol(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("_"), tag("@"), alphanumeric1))))(input)
}

pub fn section_name(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((tag("."), tag("$"), tag("_"), alphanumeric1))))(input)
}

pub fn section_rule(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((
        tag("."),
        alphanumeric1,
        tag("("),
        tag(")"),
        tag("*"),
        tag("$"),
        tag("_"),
        tag("-"),
        tag(" "),
        tag(":"),
        tag("?"),
    ))))(input)
}

pub fn path_delimiter(input: &str) -> IResult<&str, &str> {
    alt((tag("/"), tag("\\")))(input)
}

pub fn root(input: &str) -> IResult<&str, &str> {
    let windows_root = recognize(tuple((alpha1, tag(":"), path_delimiter)));
    let linux_root = path_delimiter;
    alt((windows_root, linux_root))(input)
}

pub fn directory(input: &str) -> IResult<&str, &str> {
    recognize(pair(path_name, path_delimiter))(input)
}

pub fn path_name(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((alphanumeric1, tag("."), tag("-"), tag("_")))))(input)
}

pub fn file_name(input: &str) -> IResult<&str, &str> {
    path_name(input)
}

pub fn path(input: &str) -> IResult<&str, &str> {
    let true_file_name = recognize(tuple((tag("("), path_name, tag(")"))));
    recognize(tuple((
        opt(root),
        many0(directory),
        file_name,
        opt(true_file_name),
    )))(input)
}

pub fn assignment(input: &str) -> IResult<&str, &str> {
    recognize(tuple((identifier, space0, tag("="), not_line_ending)))(input)
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
        assert!(section_rule("*mcop_copy.o(.sbss.var*.a4..*__C*_0)").is_ok());
    }

    #[test]
    fn test_assignment() {
        let input = "__image_base__ = 0x632c0000";
        let result = assignment(input);
        assert!(result.is_ok());
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
}
