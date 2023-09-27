use std::fs::File;
use std::io::{BufReader, Read, Result, Take, Error, ErrorKind};

enum FileType {
    WAV,
    RAW,
}

fn is_wav(reader: &mut Take<BufReader<File>>) -> Result<bool> {
    let mut buffer = [0u8; 12];
    reader.read_exact(&mut buffer)?;
    Ok(&buffer[..4] == b"RIFF" && &buffer[8..] == b"WAVE")
}

fn detect_file_type(filename: &str) -> Result<FileType> {
    let file = File::open(filename)?;
    let file_len = file.metadata()?.len();

    if file_len < 12 {
        return Err(Error::new(ErrorKind::InvalidData, "File is too short"));
    }

    if is_wav(&mut BufReader::new(file).take(12))? {
        Ok(FileType::WAV)
    } else {
        Ok(FileType::RAW)
    }
}