use std::{
    fs::File,
    io::{self, Read, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use chrono::Timelike;
use zip::{
    write::{FileOptions, SimpleFileOptions},
    CompressionMethod, ZipWriter,
};

use crate::{helper, Config};

use super::{file_formatter::FileFormatter, file_name::FileName};

#[derive(Clone)]
pub(crate) struct FileManager {
    file_format: FileFormatter,
    file_name: FileName,
    file_constraints: FileConstraints,
}

pub(crate) enum CompressFileError {
    UnableToCreateZipFile,
    UnableToOpenFileToCompress,
    UnableToStartZipArchiving,
    UnableToCopyContents,
    UnableToWriteToArchive,
    UnableToFinishArchivation,
    UnableToGetCompressionSettings,
    InaccessibleArchivationDirectory,
}

impl FileManager {
    pub(crate) fn init_from_string(format: String, config: Config) -> Option<FileManager> {
        let f_format = match FileFormatter::try_from_string(format) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("An error occured during parsing your format: {}", e);
                return None;
            }
        };
        let f_name = match FileName::from_file_formatter(f_format.clone(), config.level) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("An error occured during parsing your format: {}", e);
                return None;
            }
        };
        Some(FileManager {
            file_format: f_format,
            file_name: f_name,
            file_constraints: Default::default(),
        })
    }
    /// Returns full current file name (that already exists) in a String
    pub(crate) fn get_file_name(&self) -> String {
        self.file_name.get_full_file_name()
    }
    pub(crate) fn remove_rotations(&mut self) {
        self.file_constraints.rotation = Vec::new();
    }
    pub(crate) fn add_rotation(&mut self, string: String) -> bool {
        let rot_type = match RotationType::try_from_string(string) {
            Some(r) => r,
            None => {
                return false;
            }
        };
        let rot = Rotation::init_from_rotation_type(rot_type);
        self.file_constraints.rotation.push(rot);
        true
    }
    pub(crate) fn set_compression(&mut self, string: String) -> bool {
        match CompressionType::try_from_string(string) {
            Some(r) => {
                self.file_constraints.compression = Some(r);
                true
            }
            None => {
                eprintln!("Incorrect value to the compression field");
                false
            }
        }
    }
    pub(crate) fn remove_compression(&mut self) {
        self.file_constraints.compression = None;
    }

    pub(crate) fn create_new_file(&mut self, config: &Config) {
        let curr_file_name = self.file_name.get_full_file_name();
        match std::fs::exists(&curr_file_name) {
            Err(e) => {
                eprintln!("An error occured while trying to find a file: {}", e);
                return;
            }
            Ok(r) if !r => {
                let new_f_name =
                    match FileName::from_file_formatter(self.file_format.clone(), config.level) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Couldn't get file name due to the next reason: {}", e);
                            return;
                        }
                    };
                self.file_name = new_f_name;
                let f_name_str = self.file_name.get_full_file_name();
                match std::fs::File::create(f_name_str) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Couldn't create a file due to the next reason: {}", e);
                        return;
                    }
                }
            }
            _ => {
                self.file_name.increase_num();
                self.create_new_file(config);
            }
        };
    }
    fn get_path_to_compression_foler() -> String {
        "./loggit_archives/".to_string()
    }
    pub(crate) fn verify_arichive_dir() -> Result<bool, ()> {
        let folder_path = &FileManager::get_path_to_compression_foler();
        match std::fs::exists(folder_path) {
            Err(e) => {
                eprintln!("Couldn't verify the existence of the archives folder due to the next reason: {}", e);
                Err(())
            }
            Ok(r) if r => Ok(true),
            _ => match std::fs::create_dir(folder_path) {
                Ok(_) => Ok(true),
                Err(e) => {
                    eprintln!(
                        "Couldn't create an archives folder due to the next reason: {}",
                        e
                    );
                    Err(())
                }
            },
        }
    }
    fn compress_zip(&self, path: &str) -> Result<(), CompressFileError> {
        let folder_path = &FileManager::get_path_to_compression_foler();
        let path_to_zip = format!("{}/{}.zip", folder_path, path);
        let zip_file_path = Path::new(&path_to_zip);
        let zip_file = match File::create(zip_file_path) {
            Err(e) => {
                eprintln!("Couldn't create zip archive due to the next reason: {}", e);
                return Err(CompressFileError::UnableToCreateZipFile);
            }
            Ok(f) => f,
        };

        let mut zip = ZipWriter::new(zip_file);

        // Define the files you want to compress.
        let files_to_compress: Vec<PathBuf> = vec![
            PathBuf::from(path),
            // Add more files as needed
        ];

        // Set compression options (e.g., compression method)
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::DEFLATE);

        // Iterate through the files and add them to the ZIP archive.
        for file_path in &files_to_compress {
            let file = match File::open(file_path) {
                Err(e) => {
                    eprintln!(
                        "Couldn't open the file {} to compress due to the next reason: {}",
                        path, e
                    );
                    return Err(CompressFileError::UnableToOpenFileToCompress);
                }
                Ok(f) => f,
            };
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            // Adding the file to the ZIP archive.
            match zip.start_file(file_name, options) {
                Ok(r) => {}
                Err(e) => {
                    eprintln!("Couldn't start archiving due to the next reason: {}", e);
                    return Err(CompressFileError::UnableToStartZipArchiving);
                }
            };

            let mut buffer = Vec::new();
            match std::io::copy(&mut file.take(u64::MAX), &mut buffer) {
                Err(e) => {
                    eprintln!("Couldn't copy the contents from the file {} to the archive due to the next reason: {}", path, e);
                    return Err(CompressFileError::UnableToCopyContents);
                }
                Ok(_) => {}
            };

            match zip.write_all(&buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "Couldn't write all the contents of the file {} due to the next reason: {}",
                        path, e
                    );
                    return Err(CompressFileError::UnableToWriteToArchive);
                }
            };
        }

        match zip.finish() {
            Err(e) => {
                eprintln!("Couldn't finish archiving due to the next reason: {}", e);
                return Err(CompressFileError::UnableToFinishArchivation);
            }
            Ok(_) => {}
        };

        Ok(())

        //println!("Files compressed successfully to {:?}", zip_file_path);
    }
    pub(crate) fn compress_file(&self, path: &str) -> Result<(), CompressFileError> {
        if FileManager::verify_arichive_dir().is_err() {
            eprintln!("Couldn't compress file due to the error listed above!");
            return Err(CompressFileError::InaccessibleArchivationDirectory);
        }
        if let Some(compr_t) = &self.file_constraints.compression {
            match compr_t {
                CompressionType::Zip => self.compress_zip(path),
            }
        } else {
            Err(CompressFileError::UnableToGetCompressionSettings)
        }
    }
    pub(crate) fn verify_constraints(&mut self, config: &Config) {
        let curr_file_name = self.file_name.get_full_file_name();
        match std::fs::exists(&curr_file_name) {
            Err(e) => {
                eprintln!("An error occured while trying to find a file: {}", e);
                return;
            }
            Ok(r) if !r => {
                // file doesn't exist
                match File::create(&curr_file_name) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!(
                            "Couldn't create a file {} due to the next reason: {}",
                            &curr_file_name, e
                        );
                        return;
                    }
                }
            }
            _ => {}
        };
        let file = match std::fs::File::open(&curr_file_name) {
            Err(e) => {
                eprintln!(
                    "Couldn't open the file {} due to the next reason: {}",
                    &curr_file_name, e
                );
                return;
            }
            Ok(f) => f,
        };
        let f_size = match file.metadata() {
            Err(e) => {
                eprintln!(
                    "Couldn't get the file's {} metadata due to the next reason: {}",
                    &curr_file_name, e
                );
                return;
            }
            Ok(data) => data.size(),
        };
        let mut last_idx = -1;
        for idx in 0..self.file_constraints.rotation.len() {
            let rot = self.file_constraints.rotation[idx];
            match rot.rotation_type {
                RotationType::Period(_) | RotationType::Time(_, _) => {
                    let unix_now = chrono::Utc::now().timestamp() as u64;
                    if unix_now > rot.next_rotation || last_idx != -1 {
                        // if current time is ahead of our
                        // rotation that we set a new one and create
                        // a new file
                        let new_rot = Rotation::init_from_rotation_type(rot.rotation_type);
                        self.file_constraints.rotation[idx] = new_rot;
                        if last_idx == -1 {
                            self.create_new_file(config);
                            if self.compress_file(&curr_file_name).is_ok() {
                                if let Err(e) = FileManager::delete_file(&curr_file_name) {
                                    eprintln!(
                                        "Couldn't delete log file {} due to the next reason: {}",
                                        &curr_file_name, e
                                    );
                                }
                            }
                            last_idx = idx as i32;
                        }
                    }
                }
                RotationType::Size(_) => {
                    if f_size > rot.next_rotation || last_idx != -1 {
                        let new_rot = Rotation::init_from_rotation_type(rot.rotation_type);
                        self.file_constraints.rotation[idx] = new_rot;
                        if last_idx == -1 {
                            self.create_new_file(config);
                            if self.compress_file(&curr_file_name).is_ok() {
                                if let Err(e) = FileManager::delete_file(&curr_file_name) {
                                    eprintln!(
                                        "Couldn't delete log file {} due to the next reason: {}",
                                        &curr_file_name, e
                                    );
                                }
                            }
                            last_idx = idx as i32;
                        }
                    }
                }
            }
        }
    }
    pub(crate) fn delete_file(path: &str) -> io::Result<()> {
        std::fs::remove_file(path)
    }
    pub(crate) fn write_log(&mut self, mess: String, config: Config) {
        self.verify_constraints(&config);
        let f_name = self.get_file_name();
        if let Err(e) = helper::write_to_file(&f_name, &mess) {
            eprintln!(
                "Couldn't write to the file {} due to the next reason: {}",
                &f_name, e
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub(crate) enum RotationType {
    Period(u32),  // every 1 week for example
    Time(u8, u8), //every day at 12:00 for example
    Size(u64),    //500 MB for example
}

impl RotationType {
    pub(crate) fn try_from_string(text: String) -> Option<RotationType> {
        if text.contains(":") {
            // time
            let sp: Vec<&str> = text.split(":").collect();
            if sp.len() != 2 {
                return None;
            }
            let h: u8 = match sp[0].parse() {
                Ok(n) => n,
                Err(_) => return None,
            };
            let m: u8 = match sp[1].parse() {
                Ok(n) => n,
                Err(_) => return None,
            };
            if !(0..=23).contains(&h) {
                return None;
            }
            if !(0..=59).contains(&m) {
                return None;
            }
            Some(RotationType::Time(h, m))
        } else if text.ends_with(" KB")
            || text.ends_with(" MB")
            || text.ends_with(" GB")
            || text.ends_with(" TB")
        {
            //size
            let multiply_factor;
            if text.ends_with(" KB") {
                multiply_factor = 1;
            } else if text.ends_with(" MB") {
                multiply_factor = 1024;
            } else if text.ends_with(" GB") {
                multiply_factor = 1024 * 1024;
            } else if text.ends_with(" TB") {
                multiply_factor = 1024 * 1024 * 1024;
            } else {
                multiply_factor = 1;
            }

            let t_len = text.len();
            let text = &text[0..(t_len - 3)];
            let num: u64 = match text.parse() {
                Ok(n) => n,
                Err(_) => {
                    return None;
                }
            };
            Some(RotationType::Size(num * multiply_factor))
        } else if text.ends_with(" hour")
            || text.ends_with(" day")
            || text.ends_with(" week")
            || text.ends_with(" month")
            || text.ends_with(" year")
        {
            // period
            let multiply_factor;
            let finish_txt: &str = {
                if text.ends_with(" hour") {
                    multiply_factor = 60 * 60;
                    " hour"
                } else if text.ends_with(" day") {
                    multiply_factor = 60 * 60 * 24;
                    " day"
                } else if text.ends_with(" week") {
                    multiply_factor = 60 * 60 * 24 * 7;
                    " week"
                } else if text.ends_with(" month") {
                    multiply_factor = 60 * 60 * 24 * 30;
                    " month"
                } else {
                    multiply_factor = 60 * 60 * 24 * 365;
                    " year"
                }
            };
            let fin_len = finish_txt.len();
            let str_len = text.len();
            let text_to_parse = &text[0..(str_len - fin_len)];
            let num: u32 = match text_to_parse.parse() {
                Ok(n) => n,
                Err(_) => {
                    return None;
                }
            };
            Some(RotationType::Period(num * multiply_factor))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Rotation {
    rotation_type: RotationType,
    next_rotation: u64,
}
impl Rotation {
    pub(crate) fn init_from_rotation_type(rot_type: RotationType) -> Rotation {
        match rot_type {
            RotationType::Period(p) => {
                let unix_time: u64 = chrono::Utc::now().timestamp().try_into().unwrap_or(0);
                let next_to_rotate = unix_time + (p as u64);
                Rotation {
                    rotation_type: rot_type,
                    next_rotation: next_to_rotate,
                }
            }
            RotationType::Time(h, m) => {
                let now = chrono::Local::now();
                let curr_h: u8 = now.hour().try_into().unwrap_or(0);
                let curr_m: u8 = now.minute().try_into().unwrap_or(0);
                if curr_h < h || (curr_h == h && curr_m < m) {
                    // if next rotation is today
                    let unix: u64 = now.timestamp().try_into().unwrap_or(0);
                    let secs_curr = ((curr_h as u64) * 60 * 60) + ((curr_m as u64) * 60);
                    let secs_desirable = ((h as u64) * 60 * 60) + ((m as u64) * 60);
                    let diff = secs_desirable - secs_curr;
                    Rotation {
                        rotation_type: rot_type,
                        next_rotation: unix + diff,
                    }
                } else {
                    //tomorrow
                    let unix: u64 = now.timestamp().try_into().unwrap_or(0);
                    let secs_till_tomorrow =
                        (24 * 60 * 60) - (((curr_h as u64) * 60 * 60) + ((curr_m as u64) * 60));
                    let secs_desirable = ((h * 60 * 60) + (m * 60)) as u64;
                    Rotation {
                        rotation_type: rot_type,
                        next_rotation: unix + secs_till_tomorrow + secs_desirable,
                    }
                }
            }
            RotationType::Size(s) => Rotation {
                rotation_type: rot_type,
                next_rotation: s,
            },
        }
    }
}

#[derive(Clone)]
pub(crate) enum CompressionType {
    Zip,
}

impl CompressionType {
    pub(crate) fn try_from_string(text: String) -> Option<CompressionType> {
        if text == *"zip" {
            Some(CompressionType::Zip)
        } else {
            None
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct FileConstraints {
    compression: Option<CompressionType>,
    rotation: Vec<Rotation>,
}
