#[derive(Debug, PartialEq)]
pub struct Symbol<'a> {
    pub name: &'a str,
    pub address: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct Section<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub size: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct FileSection<'a> {
    pub section: &'a str,
    pub file: &'a str,
    pub address: &'a str,
    pub size: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct FileSectionGroup<'a> {
    pub file_section: FileSection<'a>,
    pub symbols: Vec<Symbol<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct SectionGroup<'a> {
    pub section: Section<'a>,
    pub file_section_groups: Vec<FileSectionGroup<'a>>,
}
