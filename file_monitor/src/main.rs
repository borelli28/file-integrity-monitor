use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{self, BufReader, BufWriter, Read};
use sha2::{Digest, Sha256};
use chrono::{Utc};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use serde_json::{json, Value};


#[derive(Serialize, Deserialize)]
struct Hashes {
    hash: String,
    file_path: String,
    timestamp: String,
}

fn gen_dirs_file() -> Result<String, io::Error> {
    let file = "./data/dirs.json";

    let data = vec![
        json!({"file_path": "/bin"}),
        json!({"file_path": "/sbin"}),
        json!({"file_path": "/usr/bin"}),
        json!({"file_path": "/usr/sbin"}),
        json!({"file_path": "/usr/local/bin"}),
        json!({"file_path": "/usr/local/sbin"}),
        json!({"file_path": "/lib"}),
        json!({"file_path": "/lib64"}),
        json!({"file_path": "/usr/lib"}),
        json!({"file_path": "/usr/lib64"}),
        json!({"file_path": "/usr/local/lib"}),
        json!({"file_path": "/usr/local/lib64"}),
        json!({"file_path": "/etc"}),
        json!({"file_path": "/boot"}),
        json!({"file_path": "/Library"}),
        json!({"file_path": "/var/log"}),
        json!({"file_path": "/etc/init.d"}),
        json!({"file_path": "/etc/launchd.conf"}),
        json!({"file_path": "/lib/modules"}),
        json!({"file_path": "/System/Library/Extensions"}),
        json!({"file_path": "/etc/cron.d"}),
        json!({"file_path": "/usr/lib/cron/tabs"}),
        json!({"file_path": "/etc/network"}),
        json!({"file_path": "/etc/security"}),
        json!({"file_path": "/etc/ssh"}),
        json!({"file_path": "/var/www"}),
        json!({"file_path": "/Library/WebServer/Documents"}),
        json!({"file_path": "/var/lib/mysql"}),
        json!({"file_path": "/var/lib/postgresql"}),
        json!({"file_path": "/usr/local/var/mysql"}),
        json!({"file_path": "/usr/local/var/postgres"}),
        json!({"file_path": "/System/Library"}),
        json!({"file_path": "/usr/libexec"})
    ];

    let json_data = serde_json::to_string_pretty(&data).unwrap();
    fs::write(file, json_data).unwrap();
    Ok(String::from("Ok"))
}

fn calculate_sha256(file_path: &str) -> Result<String, io::Error> {
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

fn hash_file(file_path: &str) -> String {
    let result = calculate_sha256(file_path);

    match result {
        Ok(hash) => {
            return hash
        }
        Err(err) => {
            return err.to_string()
        }
    }
}

fn get_hash(hash: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string("./data/hashes.json")?;

    // Parse the JSON into a serde_json Value
    let data: Value = serde_json::from_str(&contents)?;

    // Search for the object
    if let Some(array) = data.as_array() {
        for obj in array {
            if let Some(path) = obj.get("hash") {
                if let Some(obj_hash) = path.as_str() {
                    if obj_hash == hash {
                        // println!("obj_hash == hash, {}", obj_hash);
                        return Ok(obj.to_string());
                    }
                }
            }
        }
    }

    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Hash not found"))
}

fn create_file(file_path: &str) -> Result<String, io::Error> {
    let empty_array: Vec<Hashes> = Vec::new(); // Create an empty Vec of Hashes
    let json_string = serde_json::to_string(&empty_array)?; // Serialize the empty array to a JSON string
    match fs::write(file_path, json_string) {
        Ok(_) => {
            Ok(String::from("Ok"))},
        Err(err) => {
            Err(err)
        }
    }
}

fn check_file_exists(file_path: &str) -> Result<String, io::Error> {
    match fs::metadata(file_path) {
        Ok(_) => {
            Ok(String::from("Ok"))},
        Err(_) => {
            return Err(io::Error::new(io::ErrorKind::Other, "An error occurred -- check_file_exists"));
        }
    }
}

fn write_hash(hash: &str, file_path: &str, creation_timestamp: &str) -> Result<String, io::Error> {
    let hashes_file = "./data/hashes.json";
    match check_file_exists(hashes_file) {
        Ok(_) => {
            let mut hashes: Vec<Hashes> = match fs::read_to_string(hashes_file) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or(Vec::new()) // Parse the existing content into a Vec<Hashes>
                },
                Err(_) => Vec::new(), // If the file doesn't exist or is empty, create a new Vec<Hashes>
            };

            let new_hash = Hashes {
                hash: hash.to_string(),
                file_path: file_path.to_string(),
                timestamp: creation_timestamp.to_string(),
            };

            hashes.push(new_hash);
            let json_string = serde_json::to_string_pretty(&hashes)?; // Serialize the Vec back to a JSON string
            fs::write(hashes_file, json_string)?; // Write the updated JSON string back to the file

            Ok(String::from("Added to hashes.json"))
        }
        Err(_err) => {
            match create_file(hashes_file) {
                Ok(_) => {
                    let _ = write_hash(hash, file_path, creation_timestamp);
                    Ok(String::from("Ok"))
                },
                Err(err) => Err(err),
            }
        }
    }
}

fn delete_hash(hash_file_path: &str) -> Result<String, io::Error> {
    let search_for_this_path = hash_file_path;
    let file_path = "./data/hashes.json";

    let contents = fs::read_to_string(file_path)?;

    // Parse the JSON into a serde_json Value
    let mut data: Value = serde_json::from_str(&contents).expect("Error parsing JSON");

    // Search for the object and remove it if found
    if let Some(array) = data.as_array_mut() {
        array.retain(|obj| {
            if let Some(path) = obj.get("file_path") {
                if let Some(path_str) = path.as_str() {
                    return path_str != search_for_this_path;
                }
            }
            true
        });
    }

    // Write the modified JSON back to the file
    let new_contents = serde_json::to_string_pretty(&data)?;
    fs::write(file_path, new_contents)?;

    Ok(String::from("Ok"))
}

fn add_file(file_path: &str) -> Result<String, io::Error> {
    let dir_path = "./data/dirs.json";

    let mut file = OpenOptions::new().read(true).write(true).open(dir_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Parse the JSON into a serde_json Value
    let mut data: Value = serde_json::from_str(&contents)?;

    let new_object = json!({"file_path": file_path});

    // Add the new object to the array
    if let Some(array) = data.as_array_mut() {
        array.push(new_object);
    }

    // Write the modified JSON back to the file
    let file = File::create(dir_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data)?;

    Ok(String::from("Ok"))
}

fn full_scan(file_path: &str) -> Result<String, io::Error> {
    match check_file_exists(file_path) {
        Ok(_) => {
            println!("Reading directories... Please don't quit the program until it's complete.");

            let mut file = File::open(file_path).expect("File not found");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Error reading the file");

            let json_data: serde_json::Value = serde_json::from_str(&contents).expect("Error parsing JSON");

            if let Some(obj) = json_data.as_array(){
                let obj_length = obj.len();

                if obj_length > 0 {
                    for i in obj {
    
                        let line: String = i["file_path"].as_str().unwrap_or("default_path").to_string();
                        let the_path = PathBuf::from(line);
    
                        if let Ok(entries) = std::fs::read_dir(the_path) { // Return true if directory is traversable, it's found
                            for entry in entries {
                                let entry = entry?;
                                let path = entry.path();
                                if path.is_dir() {
                                    // println!("path is dir");
                                    continue
                                } else {
                                    let path = format!("{}", path.to_string_lossy()); // Convert PathBuff to str
                                    let hash = hash_file(&path);
                                    let hash_str: &str = &hash;
                                    let now = Utc::now();
                                    let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
                                    
                                    // Delete previous object from file before writing the new object
                                    let _ = delete_hash(&path);
    
                                    match write_hash(hash_str, &path, timestamp) {
                                        Ok(_) => {
                                            // println!("Write Ok");
                                            continue
                                        }
                                        Err(err) => {
                                            eprintln!("Error: {}", err);
                                        }
                                    }
                                }
                            }
                        } else { // String is a file path instead of a directory path
                            // println!("{} path is a file instead of directory, but no biggy...", i["file_path"]);
                            let _line: String = i["file_path"].as_str().unwrap_or("default_path").to_string();
                            let hash = hash_file(&_line);
                            let hash_str: &str = &hash;
                            let now = Utc::now();
                            let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();
    
                            // Delete previous object from file before writing the new object
                            let _ = delete_hash(&_line);
    
                            match write_hash(hash_str, &_line, timestamp) {
                                Ok(_) => {
                                    // println!("Write Ok");
                                    continue
                                }
                                Err(err) => {
                                    eprintln!("Error: {}", err);
                                }
                            }
                        }
                    }
                } else {
                    println!("File not found");
                }
            } else {
                println!("The parsed JSON is not an object");
            }
            Ok(String::from("Ok"))
        }
        Err(_) => {
            match gen_dirs_file() {
                Ok(_) => {
                    let _ = full_scan(file_path);
                    Ok(String::from("Ok"))
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return Err(io::Error::new(io::ErrorKind::Other, "An error occurred"));
                }
            }
        }
    }
}

fn clear_data() -> Result<String, io::Error> {
    fs::remove_file("./data/dirs.json")?;
    fs::remove_file("./data/hashes.json")?;
    Ok(String::from("Ok"))
}

fn compare_hash(hash: &str) -> Result<String, io::Error> {
    let response_str = get_hash(&hash).map_err(|_| {
        io::Error::new(io::ErrorKind::Other, format!("Hash mismatch"))
    })?;

    // Parse the JSON response into a serde_json Value
    let response_json: Value = serde_json::from_str(&response_str)?;

    // Access the "hash" field of the JSON object and compare with the provided hash
    if let Some(hash_value) = response_json.get("hash") {
        if let Some(hash_str) = hash_value.as_str() {
            if hash_str == hash {
                return Ok(String::from("No changes"));
            }
        }
    }

    Err(io::Error::new(io::ErrorKind::Other, "Hash mismatch"))
}

fn cli_menu() {
    loop {
        println!("[G] Generate Hash, [A] Add file, [H] Check Hash, [F] Full Scan, [C] Clear Data, [Q] Quit");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input: String = input.trim().to_lowercase();

        if input == "g" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let response = hash_file(&file);
            println!("\n {} \n", response);

        } else if input == "a" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            match check_file_exists(file) {
                Ok(_) => {
                    let hash = hash_file(&file);
                    let hash = hash.as_str();
                    let now = Utc::now();
                    let timestamp: &str = &now.format("%Y-%m-%d %H:%M:%S").to_string();

                    match write_hash(hash, file, timestamp) {
                        Ok(response) => {
                            println!("\n {} \n", response);
                            let _ = add_file(file);
                            println!("\n File added! \n");
                        }
                        Err(err) => {
                            eprintln!("Error reading the file: {}", err);
                        }
                    }
                }
                Err(err) => eprintln!("{}", err),
            }

        }  else if input == "h" {
            println!("\n Enter file path: ");
            let mut file = String::new();
            io::stdin().read_line(&mut file).expect("Failed to read line");
            let file: &str = file.trim();

            let hash = hash_file(file);
            let hash: &str = &hash;

            match compare_hash(hash) {
                Ok(response) => println!("{}", response),
                Err(err) => println!("{}", err),
            }

        } else if input == "f" {
            let _ = full_scan("./data/dirs.json");

        } else if input == "c" {
            let _ = clear_data();

        } else if input == "q" {
            break

        } else {
            println!("\n Invalid input \n")
        }
    }
}

fn main() {
    cli_menu();
}