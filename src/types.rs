use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Symbol<'a> {
    pub name: &'a str,
    pub address: &'a str,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Section<'a> {
    pub name: &'a str,
    pub address: &'a str,
    pub size: &'a str,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSection<'a> {
    pub section: &'a str,
    pub file: &'a str,
    pub address: &'a str,
    pub size: &'a str,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileSectionGroup<'a> {
    #[serde(borrow)]
    pub file_section: FileSection<'a>,
    #[serde(borrow)]
    pub symbols: Vec<Symbol<'a>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionGroup<'a> {
    #[serde(borrow)]
    pub section: Section<'a>,
    #[serde(borrow)]
    pub file_section_groups: Vec<FileSectionGroup<'a>>,
}

#[derive(Serialize)]
pub struct Record<'a> {
    #[serde(borrow)]
    pub symbol: &'a str,
    #[serde(borrow)]
    pub address: &'a str,
    #[serde(borrow)]
    pub file: &'a str,
    #[serde(borrow)]
    pub old_section: &'a str,
    #[serde(borrow)]
    pub new_section: &'a str,
}
