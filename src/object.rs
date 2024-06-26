use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
    vec,
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use hex::encode;
use sha1::{Digest, Sha1};

use crate::{create_path, ObjectTypes};
use crate::{hex_to_hex_byte, Repository};

mod tree;
pub use tree::{TreeNode, TreeNodeType, TreeObject};

// TODO: Choice of picking between hashing algos
// Because git itself is trying to migrate over to SHA-256 (SHA2)
#[derive(Clone, Debug)]
pub enum ObjectHeaders {
    Commit {
        fields: HashMap<String, Vec<String>>,
        order: Vec<String>,
        message: String,
    },
    Tree(TreeObject),
    Tag {
        fields: HashMap<String, Vec<String>>,
        order: Vec<String>,
        message: String,
    },
    Blob {
        data: Vec<u8>,
    },
}
impl ObjectHeaders {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Self::Blob { data } => data.clone(),
            Self::Commit {
                fields,
                order,
                message,
            }
            | Self::Tag {
                fields,
                order,
                message,
            } => {
                let mut data: Vec<String> = Vec::new();
                for key in order {
                    for value in fields.get(key).unwrap() {
                        data.push(format!("{} {}", key, value.replace('\n', "\n ")));
                    }
                }
                data.push(String::from(""));
                data.push(message.to_owned());
                data.join("\n").as_bytes().to_owned()
            }
            Self::Tree(tree) => {
                let mut data: Vec<u8> = Vec::new();
                for entry in &tree.entries {
                    data.append(&mut entry.mode.as_bytes().to_owned());
                    data.push(0x20);
                    data.append(&mut entry.path.as_bytes().to_owned());
                    data.push(0x00);
                    // Should not error here as hash should have been generated
                    data.append(&mut hex_to_hex_byte(&entry.hash).unwrap());
                }
                data
            }
            _ => Vec::new(),
        }
    }
    fn deserialize(object_type: ObjectTypes, data: Vec<u8>) -> Result<Self, String> {
        match object_type {
            ObjectTypes::Blob => Ok(Self::Blob { data }),
            ObjectTypes::Commit | ObjectTypes::Tag => {
                let data = String::from_utf8(data).unwrap();
                let lines_slice = data.split('\n').collect::<Vec<&str>>();
                let mut fields: HashMap<String, Vec<String>> = HashMap::new();
                let mut order: Vec<String> = Vec::new();
                let mut start = 0;
                while !lines_slice[start].is_empty() {
                    // Extracts key and parses value
                    let mut end = start + 1;
                    let mut intial_line = lines_slice[start].splitn(2, ' ');
                    let key = intial_line.next().unwrap();
                    let mut value = vec![intial_line.next().unwrap()];
                    while !lines_slice[end].is_empty() && lines_slice[end].starts_with(' ') {
                        value.push(lines_slice[end].strip_prefix(' ').unwrap());
                        end += 1;
                    }
                    let value = value.join("\n");
                    let key = key.to_owned();
                    fields
                        .entry(key.clone())
                        .and_modify(|values| values.push(value.clone()))
                        .or_insert(vec![value]);
                    if !order.contains(&key) {
                        order.push(key.clone());
                    }
                    start = end;
                }
                let message = lines_slice[start + 1..].join("\n");

                Ok(Self::Commit {
                    fields,
                    order,
                    message,
                })
            }
            ObjectTypes::Tree => Ok(Self::Tree(TreeObject::from_data(data)?)),
        }
    }
}
#[derive(Debug)]
pub struct Object {
    pub header: ObjectHeaders,
    pub _type: ObjectTypes,
}
impl Object {
    pub fn new(object_type: ObjectTypes, data: Vec<u8>) -> Result<Self, String> {
        Ok(Self {
            header: ObjectHeaders::deserialize(object_type.clone(), data)?,
            _type: object_type,
        })
    }
    pub fn read_from_sha(repo: &Repository, hash: String) -> Result<Self, String> {
        // TODO: hash should be computed by the object itself
        // Using SHA-1 for now
        // There have been talks to shift to SHA-2
        // TLDR: Git has implemented the necessary software, but still have not transitioned yet
        let hash: Vec<char> = hash.chars().collect();
        let directory: String = hash[..2].iter().collect();
        let filename: String = hash[2..].iter().collect();
        let object_file_path = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory, filename],
        );
        let object_file = File::open(object_file_path.clone())
            .map_err(|e| format!("Error opening file, {:?}: {}", object_file_path, e))?;
        let mut raw_file_contents: Vec<u8> = Vec::new();
        ZlibDecoder::new(object_file)
            .read_to_end(&mut raw_file_contents)
            .map_err(|_| "Error zlib decode file contents")?;
        let mut header: Vec<u8> = Vec::new();
        let header_delimiter = 0x20;
        let mut length: Vec<u8> = Vec::new();
        let length_delimiter = 0x0;
        let mut raw_file_content_iter = raw_file_contents.into_iter().peekable();
        while raw_file_content_iter.peek().is_some() {
            let b = raw_file_content_iter.next().unwrap();
            if b == header_delimiter {
                break;
            }
            header.push(b);
        }
        while raw_file_content_iter.peek().is_some() {
            let b = raw_file_content_iter.next().unwrap();
            if b == length_delimiter {
                break;
            }
            length.push(b);
        }
        let content: Vec<u8> = raw_file_content_iter.collect();

        let header = String::from_utf8(header).unwrap();
        let length = String::from_utf8(length).unwrap();

        if length.parse::<usize>().unwrap() != content.len() {
            return Err(format!(
                "Conflicting lengths found, length: {}, content_length: {}",
                length,
                content.len()
            ));
        }

        Self::new(ObjectTypes::from_string(&header), content)
    }
    pub fn calculate_hash(&self) -> Result<String, String> {
        let header = self._type.to_string();
        let data = self.header.serialize();
        let content_length = data.len().to_string();
        let final_content = [
            header.as_bytes(),
            b"\x20",
            content_length.as_bytes(),
            b"\x00",
            data.as_slice(),
        ]
        .concat();
        // DANGER
        // This is due to rust-analyzer not being able to get the correct types
        // Also the generic array does not have the correct length
        // Related GH issue: https://github.com/RustCrypto/hashes/issues/441, https://github.com/rust-lang/rust-analyzer/issues/15242
        // sha1 crate: https://docs.rs/sha1/latest/sha1/
        // SO Ans: https://stackoverflow.com/questions/59376378/how-can-i-turn-a-genericarrayt-into-an-array-of-the-same-length
        let mut hasher = Sha1::new();
        hasher.update(&final_content);
        // Should be [u8; 20]
        let hash: [u8; 20] = hasher
            .finalize()
            .as_slice()
            .try_into()
            .expect("Wrong length");
        Ok(encode(hash))
    }
    pub fn write_to_repo(&self, repo: &Repository) -> Result<String, String> {
        let header = self._type.to_string();
        let data = self.header.serialize();
        let content_length = data.len().to_string();
        let final_content = [
            header.as_bytes(),
            b"\x20",
            content_length.as_bytes(),
            b"\x00",
            data.as_slice(),
        ]
        .concat();
        let hash: Vec<char> = self.calculate_hash()?.chars().collect();

        let directory: String = hash[..2].iter().collect();
        let filename: String = hash[2..].iter().collect();
        let object_directory = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory.clone()],
        );
        let object_file_path = create_path(
            &repo.gitdir,
            vec![String::from("objects"), directory, filename],
        );
        // Create directory if not already exists
        match fs::create_dir(object_directory.clone()) {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(format!(
                    "Error creating object directory, {:?}, {}",
                    object_directory, e
                )),
            },
        }?;
        let mut object_file = File::create(object_file_path.clone())
            .map_err(|e| format!("Error creating object file, {:?}: {}", object_file_path, e))?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&final_content)
            .map_err(|_| "Error encoding data")?;
        let compressed_content = encoder
            .finish()
            .map_err(|e| format!("Error writing encoded data: {}", e))?;
        object_file
            .write_all(&compressed_content)
            .map_err(|e| format!("Error writing encoded data to file: {}", e))?;
        Ok(hash.iter().collect::<String>())
    }
}

pub fn resolve_ref(repo: &Repository, full_ref_path: &Path) -> Result<String, String> {
    let _ref = fs::read_to_string(full_ref_path)
        .unwrap()
        .strip_suffix('\n')
        .unwrap()
        .to_owned();
    if _ref.starts_with("ref: ") {
        let next_ref = _ref.strip_prefix("ref: ").unwrap();
        let next_ref_path = create_path(&repo.gitdir, vec![next_ref.to_owned()]);
        if next_ref == "refs/heads/main" && !next_ref_path.exists() {
            return Ok(String::from("Repo not initialized yet"));
        }
        resolve_ref(repo, &next_ref_path)
    } else {
        Ok(_ref)
    }
}
pub fn create_ref(repo: &Repository, name: String, hash: String) -> Result<(), String> {
    // Check if the hash we are pointing to exists
    let full_ref_path = create_path(
        &repo.gitdir,
        vec![String::from("refs"), String::from("tags"), name.clone()],
    );
    let mut ref_file =
        File::create(full_ref_path).map_err(|e| format!("Error creating ref, {}: {}", &name, e))?;
    ref_file
        .write_all(hash.as_bytes())
        .map_err(|e| format!("Failed to write into ref, {}: {}", &name, e))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_hex() {
        let sha: [u8; 20] = [
            0b11100110, 0b01110011, 0b11010001, 0b10110111, 0b11101010, 0b10100000, 0b10101010,
            0b00000001, 0b10110101, 0b10111100, 0b00100100, 0b01000010, 0b11010101, 0b01110000,
            0b10100111, 0b01100101, 0b10111101, 0b10101010, 0b11100111, 0b01010001,
        ];
        let directory = encode(&sha[0..1]);
        let filename = encode(&sha[1..]);
        assert_eq!(directory, String::from("e6"));
        assert_eq!(
            filename,
            String::from("73d1b7eaa0aa01b5bc2442d570a765bdaae751")
        );
    }

    #[test]
    fn test_starts_with() {
        assert!(" no".starts_with(' '));
        assert!("no\n".starts_with(' '));
    }

    #[test]
    fn test_deserialize_commit() {
        let data = String::from(
            "tree 29ff16c9c14e2652b22f8b78bb08a5a07930c147
parent 206941306e8a8af65b66eaaaea388a7ae24d49a0
author Thibault Polge <thibault@thb.lt> 1527025023 +0200
committer Thibault Polge <thibault@thb.lt> 1527025044 +0200
gpgsig -----BEGIN PGP SIGNATURE-----

 iQIzBAABCAAdFiEExwXquOM8bWb4Q2zVGxM2FxoLkGQFAlsEjZQACgkQGxM2FxoL
 kGQdcBAAqPP+ln4nGDd2gETXjvOpOxLzIMEw4A9gU6CzWzm+oB8mEIKyaH0UFIPh
 rNUZ1j7/ZGFNeBDtT55LPdPIQw4KKlcf6kC8MPWP3qSu3xHqx12C5zyai2duFZUU
 wqOt9iCFCscFQYqKs3xsHI+ncQb+PGjVZA8+jPw7nrPIkeSXQV2aZb1E68wa2YIL
 3eYgTUKz34cB6tAq9YwHnZpyPx8UJCZGkshpJmgtZ3mCbtQaO17LoihnqPn4UOMr
 V75R/7FjSuPLS8NaZF4wfi52btXMSxO/u7GuoJkzJscP3p4qtwe6Rl9dc1XC8P7k
 NIbGZ5Yg5cEPcfmhgXFOhQZkD0yxcJqBUcoFpnp2vu5XJl2E5I/quIyVxUXi6O6c
 /obspcvace4wy8uO0bdVhc4nJ+Rla4InVSJaUaBeiHTW8kReSFYyMmDCzLjGIu1q
 doU61OM3Zv1ptsLu3gUE6GU27iWYj2RWN3e3HE4Sbd89IFwLXNdSuM0ifDLZk7AQ
 WBhRhipCCgZhkj9g2NEk7jRVslti1NdN5zoQLaJNqSwO1MtxTmJ15Ksk3QP6kfLB
 Q52UWybBzpaP9HEd4XnR+HuQ4k2K0ns2KgNImsNvIyFwbpMUyUWLMPimaV1DWUXo
 5SBjDB/V/W2JBFR+XKHFJeFwYhj7DD/ocsGr4ZMx/lgc8rjIBkI=
 =lgTX
 -----END PGP SIGNATURE-----

Create first draft",
        )
        .as_bytes()
        .to_owned();
        let object = ObjectHeaders::deserialize(ObjectTypes::Commit, data).unwrap();
        println!("{:?}", object);
        println!("{:?}", object.serialize());
        assert!(false);
    }
}
