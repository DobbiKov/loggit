use std::{
    fmt::format,
    fs::File,
    io::{self, BufReader},
};

use chrono::Timelike;
use thiserror::Error;
use zip::{result::ZipError, write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::{
    helper::{self, WriteToFileError},
    logger::archivation,
    Config,
};

use super::{
    file_formatter::{FileFormatter, FileFormatterTryFromStringError},
    file_name::{FileName, FileNameFromFileFormatterError},
};

#[derive(Clone, Debug)]
pub(crate) struct FileManager {
    file_format: FileFormatter,
    file_name: FileName,
    file_constraints: FileConstraints,
}

#[derive(Error, Debug)]
pub enum FileManagerFromStringError {
    #[error("string parsing for the file format error: {0}")]
    FileFormatParsingError(FileFormatterTryFromStringError),
    #[error("format parsing for the file name error: {0}")]
    FileNameParsingError(FileNameFromFileFormatterError),
}

#[derive(Error, Debug)]
pub(crate) enum CompressFileError {
    #[error("error with verifying archivation folder {0}")]
    UnableToCreateArchivationFolder(std::io::Error),
    #[error("unable to create a zip file: {0}")]
    UnableToCreateZipFile(std::io::Error),
    #[error("unable to open file to compress: {0}")]
    UnableToOpenFileToCompress(std::io::Error),
    #[error("unable to start zip archiving: {0}")]
    UnableToStartZipArchiving(ZipError),
    #[error("unable to copy the contents of the file: {0}")]
    UnableToCopyContents(std::io::Error),
    #[error("unable to write to archive")]
    UnableToWriteToArchive,
    #[error("unable to finish archivation: {0}")]
    UnableToFinishArchivation(ZipError),
    #[error("unable to get compression settings")]
    UnableToGetCompressionSettings,
    #[error("inaccessible archivation directory: {0}")]
    InaccessibleArchivationDirectory(std::io::Error),
}

#[derive(Error, Debug)]
pub(crate) enum VerifyConstraintsError {
    #[error("unable to verify file existence {0}")]
    UnableToVerifyFileExistence(std::io::Error),
    #[error("unable to create file {0} {1}")]
    UnableToCreateFile(String, std::io::Error),
    #[error("unable to open file: {0} {1}")]
    UnableToOpenFile(String, std::io::Error),
    #[error("unable to get file metadata: {0} {1}")]
    UnableToGetFileMetadata(String, std::io::Error),
    #[error("unable to delete old log files: {0} {1}")]
    UnableToDeleteOldLogFile(String, std::io::Error),
    #[error("uable to compress file")]
    UnableToCompressFile,
    #[error("unable to create a new file: {0}")]
    UnableToCreateNewFile(CreateNewFileError),
}
pub(crate) enum VerifyConstraintsRes {
    ConstraintsPassed,
    NewFileCreated,
}
#[derive(Debug, Error)]
pub(crate) enum WriteLogError {
    #[error("unable to write to the file: {0}")]
    UnableToWriteToFile(WriteToFileError),
}
#[derive(Debug, Error)]
pub(crate) enum CreateNewFileError {
    #[error("unable to verify that the file exists: {0}")]
    UnableToVerifyFileExistence(std::io::Error),
    #[error("IO error occured: {0}")]
    UnableToCreateFileIO(std::io::Error),
    #[error("unable to get the file name: {0}")]
    UnableToGetFileName(FileNameFromFileFormatterError),
}

impl FileManager {
    pub(crate) fn init_from_string(
        format: &str,
        config: Config,
    ) -> Result<FileManager, FileManagerFromStringError> {
        let f_format = match FileFormatter::try_from_string(format) {
            Ok(f) => f,
            Err(e) => {
                return Err(FileManagerFromStringError::FileFormatParsingError(e));
            }
        };
        let f_name = match FileName::from_file_formatter(f_format.clone(), config.level) {
            Ok(f) => f,
            Err(e) => {
                return Err(FileManagerFromStringError::FileNameParsingError(e));
            }
        };
        Ok(FileManager {
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
    pub(crate) fn add_rotation(&mut self, string: &str) -> bool {
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
    pub(crate) fn set_compression(&mut self, string: &str) -> bool {
        match CompressionType::try_from_string(string) {
            Some(r) => {
                self.file_constraints.compression = Some(r);
                true
            }
            None => false,
        }
    }
    pub(crate) fn remove_compression(&mut self) {
        self.file_constraints.compression = None;
    }

    pub(crate) fn create_new_file(&mut self, config: &Config) -> Result<(), CreateNewFileError> {
        loop {
            match std::path::Path::new(&self.file_name.get_full_file_name()).exists() {
                false => {
                    let new_f_name =
                        match FileName::from_file_formatter(self.file_format.clone(), config.level)
                        {
                            Ok(r) => r,
                            Err(e) => {
                                return Err(CreateNewFileError::UnableToGetFileName(e));
                            }
                        };
                    self.file_name = new_f_name;
                    let f_name_str = self.file_name.get_full_file_name();
                    match std::fs::File::create(f_name_str) {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            return Err(CreateNewFileError::UnableToCreateFileIO(e));
                        }
                    }
                }
                true => {
                    self.file_name.increase_num();
                }
            }
        }
    }

    /// compresses a file by the given path in a zip archive
    fn compress_zip(&self, path: &str) -> Result<(), CompressFileError> {
        if let Err(e) = archivation::ensure_archive_dir() {
            return Err(CompressFileError::UnableToCreateArchivationFolder(e));
        }
        let zip_file_path = archivation::archive_dir().join(format!("{}.zip", path));
        let zip_file = std::fs::File::create(&zip_file_path)
            .map_err(CompressFileError::UnableToCreateZipFile)?;
        let mut zip = ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::DEFLATE);

        let file =
            std::fs::File::open(path).map_err(CompressFileError::UnableToOpenFileToCompress)?;
        let mut reader = BufReader::new(file);

        let entry_name = std::path::Path::new(path).file_name().unwrap_or_default().to_string_lossy();
        zip.start_file(entry_name, options)
            .map_err(CompressFileError::UnableToStartZipArchiving)?;
        std::io::copy(&mut reader, &mut zip).map_err(CompressFileError::UnableToCopyContents)?;
        zip.finish()
            .map_err(CompressFileError::UnableToFinishArchivation)?;
        Ok(())

        //println!("Files compressed successfully to {:?}", zip_file_path);
    }
    /// Compresses a file by the given path depending on the set compression algortithm in the
    /// config
    pub(crate) fn compress_file(&self, path: &str) -> Result<(), CompressFileError> {
        if let Err(e) = archivation::ensure_archive_dir() {
            return Err(CompressFileError::InaccessibleArchivationDirectory(e));
        }
        if let Some(compr_t) = &self.file_constraints.compression {
            match compr_t {
                CompressionType::Zip => self.compress_zip(path),
            }
        } else {
            Err(CompressFileError::UnableToGetCompressionSettings)
        }
    }
    /// verifying file constraints (rotation time and file size) and if one of the constraints
    /// doesn't pass, it creates new file (archives the changed file if it's set in the config)
    pub(crate) fn verify_constraints(
        &mut self,
        config: &Config,
    ) -> Result<VerifyConstraintsRes, VerifyConstraintsError> {
        let curr_file_name = self.file_name.get_full_file_name();
        if !std::path::Path::new(&curr_file_name).exists() {
            // file doesn't exist
            match File::create(&curr_file_name) {
                Ok(_) => {}
                Err(e) => {
                    return Err(VerifyConstraintsError::UnableToCreateFile(
                        curr_file_name.clone(),
                        e,
                    ));
                }
            }
        };
        let file = match std::fs::File::open(&curr_file_name) {
            Err(e) => {
                return Err(VerifyConstraintsError::UnableToOpenFile(
                    curr_file_name.clone(),
                    e,
                ));
            }
            Ok(f) => f,
        };
        let f_size = match file.metadata() {
            Err(e) => {
                return Err(VerifyConstraintsError::UnableToGetFileMetadata(
                    curr_file_name.clone(),
                    e,
                ));
            }
            Ok(data) => data.len(),
        };
        let mut last_idx: i32 = -1;
        // we need last_idx for: if we found not satsfying constraint, than we create a new file,
        // thus we have to update all the constraints we had, to set the to the original values,
        // as a consequence, we have last_idx, if it's not -1, than on last_idx rotation we created
        // new file and update all the constraints to initial values
        let mut idx: usize = 0;
        let mut res: Result<VerifyConstraintsRes, VerifyConstraintsError> =
            Ok(VerifyConstraintsRes::ConstraintsPassed);
        loop {
            if (idx) >= (self.file_constraints.rotation.len()) && last_idx == -1 {
                // if we haven't
                // met any
                // unverified
                // constraints
                // and reached
                // the end, we
                // stop
                break;
            }
            if (idx as i32) == last_idx {
                // if we reached last index it means we restarted from 0,
                // then we ran through the right part from last_idx and
                // though the left as well
                break;
            }
            if idx >= (self.file_constraints.rotation.len()) && last_idx != -1 {
                // if we meet the
                // end and
                // last_idx is not
                // -1, then we
                // should go
                // through the
                // left part from
                // last_idx again
                idx = 0;
            }
            if last_idx == 0 {
                // if last_idx == 0 then the first one wasn't satisfied and it was
                // immediately handled
                break;
            }

            //each rot logic

            let rot = self.file_constraints.rotation[idx];
            match rot.rotation_type {
                RotationType::Period(_) | RotationType::Time(_, _) => {
                    let unix_now = chrono::Utc::now().timestamp()        
                            .max(0) // never negative
                            as u64;
                    if unix_now > rot.next_rotation || last_idx != -1 {
                        // if current time is ahead of our
                        // rotation that we set a new one and create
                        // a new file
                        let new_rot = Rotation::init_from_rotation_type(rot.rotation_type);
                        self.file_constraints.rotation[idx] = new_rot;
                        if last_idx == -1 {
                            match self.create_new_file(config) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(VerifyConstraintsError::UnableToCreateNewFile(e));
                                }
                            }
                            if self.compress_file(&curr_file_name).is_ok() {
                                if let Err(e) = FileManager::delete_file(&curr_file_name) {
                                    res = Err(VerifyConstraintsError::UnableToDeleteOldLogFile(
                                        curr_file_name.clone(),
                                        e,
                                    ));
                                } else {
                                    res = Ok(VerifyConstraintsRes::NewFileCreated)
                                }
                            } else {
                                res = Err(VerifyConstraintsError::UnableToCompressFile)
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
                            match self.create_new_file(config) {
                                Ok(_) => {}
                                Err(e) => {
                                    return Err(VerifyConstraintsError::UnableToCreateNewFile(e));
                                }
                            }
                            if self.compress_file(&curr_file_name).is_ok() {
                                if let Err(e) = FileManager::delete_file(&curr_file_name) {
                                    res = Err(VerifyConstraintsError::UnableToDeleteOldLogFile(
                                        curr_file_name.clone(),
                                        e,
                                    ));
                                } else {
                                    res = Ok(VerifyConstraintsRes::NewFileCreated)
                                }
                            } else {
                                res = Err(VerifyConstraintsError::UnableToCompressFile)
                            }
                            last_idx = idx as i32;
                        }
                    }
                }
            }
            // end
            idx += 1;
        }
        res
    }
    pub(crate) fn delete_file(path: &str) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    pub(crate) fn write_log(
        &mut self,
        mess: &str,
        config: Config,
    ) -> Result<VerifyConstraintsRes, WriteLogError> {
        let mut ok_res = Ok(VerifyConstraintsRes::ConstraintsPassed);
        match self.verify_constraints(&config) {
            Ok(r) => ok_res = Ok(r),
            Err(e) => {
                eprintln!("An error occured while verifying constraints: {}", e);
                eprintln!("Trying to write to an old file");
                ok_res = Err(e)
            }
        }
        let f_name = self.get_file_name();

        helper::write_to_file(&f_name, mess)
            .map(|_| ok_res.unwrap())
            .map_err(WriteLogError::UnableToWriteToFile)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub(crate) enum RotationType {
    Period(u64),  // every 1 week for example
    Time(u8, u8), //every day at 12:00 for example
    Size(u64),    //500 MB for example
}

impl RotationType {
    pub(crate) fn try_from_string(text: &str) -> Option<RotationType> {
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
                multiply_factor = 1024;
            } else if text.ends_with(" MB") {
                multiply_factor = 1024 * 1024;
            } else if text.ends_with(" GB") {
                multiply_factor = 1024 * 1024 * 1024;
            } else if text.ends_with(" TB") {
                multiply_factor = 1024 * 1024 * 1024 * 1024;
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
            let multiply_factor: u64;
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
            let num: u64 = match text_to_parse.parse() {
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

#[derive(Clone, Copy, Debug)]
pub(crate) struct Rotation {
    rotation_type: RotationType,
    next_rotation: u64,
}
impl Rotation {
    pub(crate) fn init_from_rotation_type(rot_type: RotationType) -> Rotation {
        match rot_type {
            RotationType::Period(p) => {
                let unix_time: u64 = chrono::Utc::now().timestamp().try_into().unwrap_or(0);
                let next_to_rotate = unix_time + p;
                Rotation {
                    rotation_type: rot_type,
                    next_rotation: next_to_rotate,
                }
            }
            RotationType::Time(h, m) => {
                let h = h as u64;
                let m = m as u64;
                let now = chrono::Local::now();
                let curr_h: u64 = now.hour().into();
                let curr_m: u64 = now.minute().into();
                if curr_h < h || (curr_h == h && curr_m < m) {
                    // if next rotation is today
                    let unix: u64 = now.timestamp().max(0) as u64;
                    let secs_curr = ((curr_h) * 60 * 60) + (curr_m * 60);
                    let secs_desirable = (h * 60 * 60) + (m * 60);
                    let diff = secs_desirable - secs_curr;
                    Rotation {
                        rotation_type: rot_type,
                        next_rotation: unix + diff,
                    }
                } else {
                    //tomorrow
                    let unix: u64 = now.timestamp().max(0) as u64;
                    let secs_till_tomorrow =
                        (24 * 60 * 60) - ((curr_h * 60 * 60) + (curr_m * 60));
                    let secs_desirable = ((h * 60 * 60) + (m * 60));
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

#[derive(Clone, Debug)]
pub(crate) enum CompressionType {
    Zip,
}

impl CompressionType {
    pub(crate) fn try_from_string(text: &str) -> Option<CompressionType> {
        if text == "zip" {
            Some(CompressionType::Zip)
        } else {
            None
        }
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct FileConstraints {
    compression: Option<CompressionType>,
    rotation: Vec<Rotation>,
}
