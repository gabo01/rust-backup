extern crate artid_core as app;
extern crate chrono;
extern crate serde_json as json;
extern crate tempfile;

use app::prelude::{BackupOptions, ConfigFile};
use chrono::offset::Utc;
use std::fs::{self, File};
use std::io::Write;

macro_rules! tmpdir {
    () => {
        tempfile::tempdir().expect("Unable to create tmp directory");
    };
}

macro_rules! tmppath {
    ($dir:expr, $path:expr) => {
        $dir.path().join($path)
    };
}

macro_rules! create_file {
    ($path:expr) => {
        {
            let _file = File::create($path).expect("Unable to create file");
            $path
        }
    };

    ($path:expr, $($arg:tt)*) => {
        {
            use std::io::Write;

            let mut file = File::create($path).expect("Unable to create file");
            write!(file, $($arg)*).expect("Unable to write to file");
            $path
        }
    }
}

macro_rules! read_file {
    ($file:expr) => {{
        use std::io::Read;

        let mut file = File::open($file).expect("Unable to open file");
        let mut buf = String::new();
        file.read_to_string(&mut buf).expect("Unable to read file");
        buf
    }};
}

macro_rules! rfc3339 {
    ($stamp:expr) => {{
        use chrono::SecondsFormat;
        $stamp.to_rfc3339_opts(SecondsFormat::Nanos, true)
    }};
}

#[test]
fn test_config_file_load_valid() {
    let dir = tmpdir!();
    create_file!(
        tmppath!(dir, "config.json"),
        "[
        {{
            \"path\": \"asd\", 
            \"origin\": \"$HOME\", 
            \"modified\": null
        }}
    ]"
    );
    assert!(ConfigFile::load_from(dir, "config.json").is_ok());
}

#[test]
fn test_config_file_load_valid_with_modified() {
    let dir = tmpdir!();
    create_file!(
        tmppath!(dir, "config.json"),
        "[
        {{
            \"path\": \"asd\", 
            \"origin\": \"$HOME\", 
            \"modified\": [\"{}\"]
        }}
    ]",
        rfc3339!(Utc::now())
    );
    assert!(ConfigFile::load_from(dir, "config.json").is_ok());
}

#[test]
fn test_config_file_load_invalid() {
    let dir = tmpdir!();
    create_file!(
        tmppath!(dir, "config.json"),
        "[
        {{
            \"path\": \"asd, 
            \"origin\": \"$HOME\", 
            \"modified\": null
        }}
    ]"
    );
    assert!(ConfigFile::load_from(dir, "config.json").is_err());
}

#[test]
fn test_config_load() {
    let tmp = tmpdir!();
    fs::create_dir(tmppath!(tmp, ".backup")).expect("Unable to create folder");
    create_file!(
        tmppath!(tmp, ".backup/config.json"),
        "[
        {{
            \"path\": \"backup\",
            \"origin\": \"{}\",
            \"modified\": null
        }}
    ]",
        tmppath!(tmp, "origin").display().to_string()
    );
    assert!(ConfigFile::load(tmp.path()).is_ok());
}

#[test]
fn test_config_load_from() {
    let tmp = tempfile::tempdir().unwrap();

    let mut file = File::create(tmp.path().join("config.json")).unwrap();
    write!(
        file,
        "[
        {{
            \"path\": \"backup\",
            \"origin\": \"{}\",
            \"modified\": null
        }}
    ]",
        tmp.path().join("origin").display().to_string()
    )
    .unwrap();

    let _config = ConfigFile::load_from(tmp.path(), "config.json").unwrap();
}

#[test]
fn test_config_file_save_exists() {
    let dir = tmpdir!();
    assert!(create_file!(tmppath!(dir, "config.json")).exists());

    let config = ConfigFile::new(dir.path());
    assert!(config.save_to("config.json").is_ok());
}

#[test]
fn test_config_file_save_unexistant() {
    let dir = tmpdir!();

    let config = ConfigFile::new(dir.path());
    assert!(config.save_to("config.json").is_ok());
}

#[test]
fn test_config_save_to_format() {
    let tmp = tmpdir!();
    create_file!(
        tmppath!(tmp, "config.json"),
        "[
        {{
            \"path\": \"backup\",
            \"origin\": \"{}\",
            \"modified\": null
        }}
    ]",
        tmppath!(tmp, "origin").display().to_string()
    );

    let config = ConfigFile::load_from(tmp.path(), "config.json").expect("Unable to load file");
    config
        .save_to("config2.json")
        .expect("Unable to save the file");

    assert_eq!(
        json::to_string_pretty(config.folders()).expect("Cannot fail serialization"),
        read_file!(tmppath!(tmp, "config2.json")),
    );
}

#[test]
fn test_config_save() {
    let tmp = tmpdir!();
    fs::create_dir(tmppath!(tmp, ".backup")).expect("Unable to create folder");

    let config = ConfigFile::new(tmp.path());
    config.save().expect("Unable to save");

    assert!(tmppath!(tmp, ".backup/config.json").exists());
}

#[test]
fn test_config_backup() {
    let tmp = tmpdir!();
    let backup = tmppath!(tmp, "backup");

    fs::create_dir(tmppath!(tmp, "origin")).expect("Unable to create path");
    fs::create_dir_all(backup.join(".backup")).expect("Unable to create path");

    create_file!(
        backup.join(".backup/config.json"),
        "[
        {{
            \"path\": \"backup\",
            \"origin\": \"{origin}\",
            \"modified\": null
        }},

        {{
            \"path\": \"other\",
            \"origin\": \"{origin}\",
            \"modified\": null
        }}
    ]",
        origin = tmppath!(tmp, "origin").display().to_string()
    );

    let mut config = ConfigFile::load(&backup).expect("Unable to load file");
    let stamp = config
        .backup(BackupOptions::new(true))
        .expect("Unable to perform backup");

    assert!(backup.join(format!("backup/{}", rfc3339!(stamp))).exists());
    assert!(backup.join(format!("other/{}", rfc3339!(stamp))).exists());
}
