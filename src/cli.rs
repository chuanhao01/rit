use clap::ValueEnum;

#[derive(Clone, Debug, ValueEnum)]
pub enum ObjectTypes {
    Commit,
    Tree,
    Tag,
    Blob,
}
#[allow(clippy::inherent_to_string)]
impl ObjectTypes {
    pub fn from_string(s: &str) -> Self {
        match s {
            "commit" => Self::Commit,
            "blob" => Self::Blob,
            "tag" => Self::Tag,
            "tree" => Self::Tree,
            _ => panic!("Unknown object header used, {}", s),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Self::Commit => String::from("commit"),
            Self::Blob => String::from("blob"),
            Self::Tag => String::from("tag"),
            Self::Tree => String::from("tree"),
        }
    }
}
