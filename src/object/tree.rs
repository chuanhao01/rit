use std::fmt::Display;

use hex::encode;

#[derive(Debug, Clone)]
pub enum TreeNodeType {
    Blob,
    Commit,
    Tree,
}
impl TreeNodeType {
    fn from_string(mode: String) -> Result<Self, String> {
        // TODO: Not sure if there is any edge cases with unknown modes
        match &mode[..2] {
            "04" => Ok(Self::Tree),
            "10" => Ok(Self::Blob),
            "12" => Ok(Self::Blob),
            "16" => Ok(Self::Commit),
            _ => Err(format!("Unknown mode given, {:?}", mode)),
        }
    }
}
impl Display for TreeNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Blob => "blob",
            Self::Commit => "commit",
            Self::Tree => "tree",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub mode: String,
    pub path: String,
    pub hash: String,
    pub _type: TreeNodeType,
}
impl TreeNode {
    fn new(mode: String, path: String, hash: String) -> Result<Self, String> {
        let mut mode = mode;
        // If the mode given is a folder (which is only 5 bytes/chars long)
        if mode.len() < 6 {
            mode = String::from("0") + &mode;
        }
        Ok(Self {
            mode: mode.clone(),
            path,
            hash,
            _type: TreeNodeType::from_string(mode)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct TreeObject {
    pub entries: Vec<TreeNode>,
}
impl TreeObject {
    pub fn from_data(data: Vec<u8>) -> Result<Self, String> {
        let mut data = data.into_iter().peekable();
        let mut entries: Vec<TreeNode> = Vec::new();
        while data.peek().is_some() {
            let mode = String::from_utf8(
                data.by_ref()
                    .take_while(|&byte| byte != 0x20)
                    .collect::<Vec<u8>>(),
            )
            .unwrap();
            let path = String::from_utf8(
                data.by_ref()
                    .take_while(|&byte| byte != 0x00)
                    .collect::<Vec<u8>>(),
            )
            .unwrap();
            let hash = encode(data.by_ref().take(20).collect::<Vec<u8>>());
            entries.push(TreeNode::new(mode, path, hash)?);
        }
        entries.sort_by(|a, b| {
            let process_path = |node: &TreeNode| -> String {
                match node._type {
                    TreeNodeType::Tree => node.path.clone() + "\\",
                    _ => node.path.clone(),
                }
            };
            process_path(a).cmp(&process_path(b))
        });
        Ok(Self { entries })
    }
}
