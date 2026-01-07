//! Module that deals with packed cmp files, i.e. files that have
//! other files embedded inside them.

use std::{io::Write, path::PathBuf};

use crate::buffer::{BufferReader, BufferWriter};

const KEY: &[u8; 32] = b"\x0C\x40\x55\x0C\x2D\x41\x62\x2D\x03\x06\x48\x1E\x05\x48\x14\x05\x30\x32\x33\x34\x63\x63\x46\x33\x18\x09\x28\x0F\x06\x22\x39\x17";

pub fn unpack(file_path: &PathBuf, to_directory: &PathBuf, assume_yes: bool) -> crate::Result<()> {
    let file = std::fs::read(file_path).map_err(|e| format!("Failed to open .cmp file: {e}"))?;

    let mut reader = BufferReader::new(&file);
    let file_count = reader.read_u32()?;

    if !assume_yes {
        print!(
            "{file_count} files will be extracted to {}. Continue? (Y/n): ",
            to_directory.display()
        );
        std::io::stdout().flush()?;

        let response = {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf)?;
            buf.trim_ascii().to_lowercase()
        };

        if !response.is_empty() && response.to_lowercase() != "y" {
            println!("Aborted");
            return Ok(());
        }
    }

    for _ in 0..file_count {
        let encrypted_file_path_length = reader.read_u32()? as usize;
        let encrypted_file_path = reader.read_bytes(encrypted_file_path_length)?;

        let decrypted_file_path = decrypt_file_path(encrypted_file_path)
            .map_err(|e| format!("Failed to decrypt file path: {e}"))?;
        reader.skip(1); // NUL

        let file_offset = reader.read_u32()? as usize;

        let file_size = reader.read_u32()? as usize;

        let _unknown = reader.read_u32()?;

        let dir = decrypted_file_path
            .parent()
            .expect("decrypted path must have a parent directory");
        std::fs::create_dir_all(dir)?;

        println!("Unpacked {}", decrypted_file_path.display());

        std::fs::write(
            to_directory.join(decrypted_file_path),
            &file[file_offset..file_offset + file_size],
        )
        .map_err(|e| format!("Failed to write unpacked file to a file: {e}"))?;
    }

    Ok(())
}

pub fn pack(directory: &PathBuf, output: PathBuf) -> crate::Result<()> {
    let base_dir = directory.to_string_lossy().to_string();
    let mut file_paths = Vec::new();
    let mut files = Vec::new();

    let mut current_offset = 4u32;
    let mut stack = Vec::new();
    stack.push(directory.clone());

    while let Some(parent) = stack.pop() {
        if parent.is_dir() {
            let mut dir = std::fs::read_dir(&parent)?.collect::<Result<Vec<_>, _>>()?;

            dir.sort_unstable_by_key(|v| v.path().to_string_lossy().to_lowercase());

            for entry in dir.iter().rev() {
                stack.push(entry.path());
            }

            continue;
        }

        if parent.is_file() {
            let file = std::fs::read(&parent)?;

            let converted_path = parent
                .strip_prefix(&base_dir)?
                .to_string_lossy()
                .to_string()
                .replace("/", "\\");

            println!("Packing {}", converted_path);
            let encrypted_path = encrypt_file_path(&converted_path);

            // + 4 for path length
            // + encrypted path itself
            // + 4 for offset
            // + 4 for size
            // + 4 for unknown u32 value
            current_offset += encrypted_path.len() as u32 + 16;

            files.push(file);
            file_paths.push(encrypted_path);
        }
    }

    let mut output_file = BufferWriter::new();
    output_file.write_u32(files.len() as u32);

    for (file_path, bytes) in std::iter::zip(file_paths, &files) {
        output_file.write_u32(file_path.len() as u32 - 1);
        output_file.write_bytes(&file_path);
        output_file.write_u32(current_offset);

        let file_size = bytes.len() as u32;
        output_file.write_u32(file_size);

        current_offset += file_size;

        // Unknown.
        output_file.write_u32(0);
    }

    for file in files {
        output_file.write_bytes(&file);
    }

    std::fs::write(output, output_file.finish())?;
    Ok(())
}

fn decrypt_file_path(encrypted_buffer: &[u8]) -> crate::Result<PathBuf> {
    let mut decrypted_buffer = encrypted_buffer.to_vec();

    for (i, byte) in decrypted_buffer.iter_mut().enumerate() {
        *byte = !*byte ^ KEY[i % 32];
    }

    let path = String::from_utf8(decrypted_buffer)?;
    Ok(path.split('\\').collect())
}

fn encrypt_file_path(path: &str) -> Vec<u8> {
    let mut encrypted_buffer = path.as_bytes().to_vec();

    for (i, byte) in encrypted_buffer.iter_mut().enumerate() {
        *byte = !(*byte ^ KEY[i % 32]);
    }

    encrypted_buffer.push(b'\0');
    encrypted_buffer
}
