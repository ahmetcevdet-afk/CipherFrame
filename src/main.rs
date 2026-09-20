use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::Rng;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

const CHUNK_SIZE: usize = 1024 * 1024; // 1 MB
const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

fn main() {
    print_banner();

    println!("{CYAN}[1] Encrypt video{RESET}");
    println!("{CYAN}[2] Decrypt video{RESET}");
    println!();

    print!("Select an option: ");

    io::stdout().flush().unwrap();

    let mut choice = String::new();

    io::stdin()
        .read_line(&mut choice)
        .expect("Failed to read input");

    match choice.trim() {
        "1" => encrypt_video(),
        "2" => decrypt_video(),
        _ => {
            println!("{RED}[-] Invalid option.{RESET}");
        }
    }
}

// --------------------------------------------------
// Banner
// --------------------------------------------------

fn print_banner() {
    println!("{GREEN}");
    println!(" ██████╗██╗██████╗ ██╗  ██╗███████╗██████╗ ");
    println!("██╔════╝██║██╔══██╗██║  ██║██╔════╝██╔══██╗");
    println!("██║     ██║██████╔╝███████║█████╗  ██████╔╝");
    println!("██║     ██║██╔═══╝ ██╔══██║██╔══╝  ██╔══██╗");
    println!("╚██████╗██║██║     ██║  ██║███████╗██║  ██║");
    println!(" ╚═════╝╚═╝╚═╝     ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝");
    println!("{RESET}");

    println!("{CYAN}CipherFrame Video Encryption System{RESET}");
    println!();
}

// --------------------------------------------------
// Encrypt
// --------------------------------------------------

fn encrypt_video() {
    println!("{CYAN}=== ENCRYPT VIDEO ==={RESET}");
    println!();

    let video_path = ask_for_file("Enter the path to the video file:");

    let output_dir =
        ask_for_directory("Enter the folder where encrypted chunks should be saved:");

    println!();
    println!("{CYAN}[>] Loading video...{RESET}");

    let video_data = match fs::read(&video_path) {
        Ok(data) => data,
        Err(_) => {
            println!("{RED}[-] Failed to read video file.{RESET}");
            return;
        }
    };

    println!("{GREEN}[+] Video loaded successfully!{RESET}");
    println!("    Size: {} bytes", video_data.len());
    println!();

    // Generate a random 256-bit key.
    let mut key_bytes = [0u8; KEY_SIZE];
    rand::rng().fill(&mut key_bytes);

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    println!("{CYAN}[>] Encrypting video...{RESET}");
    println!();

    let mut chunk_number = 0;
    let mut position = 0;

    while position < video_data.len() {
        let end = std::cmp::min(
            position + CHUNK_SIZE,
            video_data.len(),
        );

        let chunk = &video_data[position..end];

        chunk_number += 1;

        // Generate a unique nonce.
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::rng().fill(&mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted_chunk = match cipher.encrypt(nonce, chunk) {
            Ok(data) => data,
            Err(_) => {
                println!(
                    "{RED}[-] Failed to encrypt chunk {}.{RESET}",
                    chunk_number
                );
                return;
            }
        };

        // File format:
        // [ nonce ][ ciphertext + authentication tag ]

        let mut output_data = Vec::with_capacity(
            NONCE_SIZE + encrypted_chunk.len(),
        );

        output_data.extend_from_slice(&nonce_bytes);
        output_data.extend_from_slice(&encrypted_chunk);

        let filename = format!("chunk_{:04}.enc", chunk_number);

        let output_path =
            Path::new(&output_dir).join(&filename);

        match fs::write(&output_path, output_data) {
            Ok(_) => {
                println!(
                    "{GREEN}[+] {} encrypted.{RESET}",
                    filename
                );
            }

            Err(_) => {
                println!(
                    "{RED}[-] Failed to save {}.{RESET}",
                    filename
                );
                return;
            }
        }

        position = end;
    }

    println!();
    println!("{GREEN}[+] Encryption completed!{RESET}");
    println!();

    println!("{CYAN}Your encryption key:{RESET}");
    println!();

    println!("{GREEN}{}{RESET}", bytes_to_hex(&key_bytes));

    println!();
    println!(
        "{RED}IMPORTANT: Keep this key safe.{RESET}"
    );
    println!(
        "{RED}You will need this key to decrypt the video.{RESET}"
    );
}

// --------------------------------------------------
// Decrypt
// --------------------------------------------------

fn decrypt_video() {
    println!("{CYAN}=== DECRYPT VIDEO ==={RESET}");
    println!();

    let encrypted_dir =
        ask_for_directory("Enter the encrypted chunks folder:");

    let key_string =
        ask_for_key("Enter your encryption key:");

    let key_bytes = match hex_to_bytes(&key_string) {
        Some(key) if key.len() == KEY_SIZE => key,
        _ => {
            println!(
                "{RED}[-] Invalid key. The key must be 64 hexadecimal characters.{RESET}"
            );
            return;
        }
    };

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let output_path =
        ask_for_output_file("Enter the output video path:");

    println!();
    println!("{CYAN}[>] Searching for encrypted chunks...{RESET}");

    let mut chunk_number = 1;
    let mut decrypted_video = Vec::new();

    loop {
        let filename =
            format!("chunk_{:04}.enc", chunk_number);

        let chunk_path =
            Path::new(&encrypted_dir).join(&filename);

        if !chunk_path.exists() {
            break;
        }

        let encrypted_data = match fs::read(&chunk_path) {
            Ok(data) => data,
            Err(_) => {
                println!(
                    "{RED}[-] Failed to read {}.{RESET}",
                    filename
                );
                return;
            }
        };

        if encrypted_data.len() < NONCE_SIZE {
            println!(
                "{RED}[-] {} is corrupted.{RESET}",
                filename
            );
            return;
        }

        // Extract nonce.
        let nonce_bytes =
            &encrypted_data[..NONCE_SIZE];

        // Extract ciphertext + authentication tag.
        let ciphertext =
            &encrypted_data[NONCE_SIZE..];

        let nonce =
            Nonce::from_slice(nonce_bytes);

        let decrypted_chunk =
            match cipher.decrypt(nonce, ciphertext) {
                Ok(data) => data,

                Err(_) => {
                    println!();
                    println!(
                        "{RED}[-] Decryption failed for {}.{RESET}",
                        filename
                    );
                    println!(
                        "{RED}[-] Invalid key or corrupted data.{RESET}"
                    );
                    return;
                }
            };

        decrypted_video.extend_from_slice(&decrypted_chunk);

        println!(
            "{GREEN}[+] {} decrypted.{RESET}",
            filename
        );

        chunk_number += 1;
    }

    if chunk_number == 1 {
        println!(
            "{RED}[-] No encrypted chunks were found.{RESET}"
        );
        return;
    }

    println!();
    println!(
        "{GREEN}[+] {} chunks decrypted successfully.{RESET}",
        chunk_number - 1
    );

    match fs::write(&output_path, decrypted_video) {
        Ok(_) => {
            println!(
                "{GREEN}[+] Video successfully restored!{RESET}"
            );
            println!("    Output: {}", output_path);
        }

        Err(error) => {
            println!(
                "{RED}[-] Failed to write the output video. {}{RESET}",
                error
            );
        }
    }
}

// --------------------------------------------------
// Input helpers
// --------------------------------------------------

fn ask_for_file(message: &str) -> String {
    loop {
        println!("{GREEN}{message}{RESET}");

        let input = read_input();

        if Path::new(&input).is_file() {
            println!(
                "{GREEN}[+] File found!{RESET}"
            );

            return input;
        }

        println!(
            "{RED}[-] File not found. Please try again.{RESET}"
        );

        println!();
    }
}

fn ask_for_directory(message: &str) -> String {
    loop {
        println!("{GREEN}{message}{RESET}");

        let input = read_input();

        if input.is_empty() {
            println!(
                "{RED}[-] Path cannot be empty.{RESET}"
            );

            continue;
        }

        let path = Path::new(&input);

        if !path.exists() {
            match fs::create_dir_all(path) {
                Ok(_) => {
                    println!(
                        "{GREEN}[+] Folder created!{RESET}"
                    );

                    return input;
                }

                Err(_) => {
                    println!(
                        "{RED}[-] Could not create folder.{RESET}"
                    );
                }
            }
        } else if path.is_dir() {
            return input;
        } else {
            println!(
                "{RED}[-] The path is not a directory.{RESET}"
            );
        }

        println!();
    }
}

fn ask_for_output_file(message: &str) -> String {
    println!("{GREEN}{message}{RESET}");

    read_input()
}

fn ask_for_key(message: &str) -> String {
    println!("{GREEN}{message}{RESET}");

    read_input()
}

fn read_input() -> String {
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().trim_matches('"').to_string()
}

// --------------------------------------------------
// Hex encoding
// --------------------------------------------------

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect()
}

// --------------------------------------------------
// Hex decoding
// --------------------------------------------------

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    let chars: Vec<char> = hex.chars().collect();

    for i in (0..chars.len()).step_by(2) {
        let high = chars[i].to_digit(16)?;
        let low = chars[i + 1].to_digit(16)?;

        bytes.push((high * 16 + low) as u8);
    }

    Some(bytes)
}