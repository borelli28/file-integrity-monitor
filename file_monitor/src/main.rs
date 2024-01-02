use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::Utc;


fn calculate_sha256(file_path: &str) -> Result<String, std::io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hash = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }

    let result = hash.finalize();
    Ok(format!("{:x}", result))
}

#[derive(Serialize, Deserialize)]
struct HashStorage {
    hashes: HashMap<String, String>,
    file_path: String,
}

impl HashStorage {
    fn new(file_path: String) -> io::Result<Self> {
        let storage = HashStorage {
            hashes: HashMap::new(),
            file_path: file_path,
        };

        Ok(storage)
    }

    fn save_to_file(&self) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(&self.hashes)?;

        let mut file = File::create(&self.file_path)?;
        file.write_all(json_data.as_bytes())?;

        Ok(())
    }

    fn add_hash(&mut self, file_path: &str) -> io::Result<()> {
        match calculate_sha256(file_path) {
            Ok(hash) => {
                self.hashes.insert(file_path.to_string(), hash);
                self.save_to_file()?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

fn hash_file(file_path: &str) {
    let result = calculate_sha256(file_path);

    match result {
        Ok(hash) => {
            println!("Hash: {}", hash);
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
        }
    }
}

// fn monitor(file_path: &str) -> io::Result<()> {
//     let directories_file = file_path;
// }

fn main() {
    let file_path = String::from("./data/hashes.json");
    HashStorage::new(file_path).expect("Failed to create HashStorage");

    let hash_this_file = "./test.txt";
    hash_file(hash_this_file);
}