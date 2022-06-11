use std::{error::Error, fs, path::PathBuf};

use serde::Deserialize;

pub mod error;
pub mod lex;
pub mod load;
pub mod parse;
pub mod validate;

pub use error::{HldrError, HldrErrorKind};

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Options {
    #[serde(default = "default_data_file")]
    pub data_file: PathBuf,

    #[serde(default)]
    pub database_conn: String,
}

impl Options {
    pub fn new(filepath: &PathBuf) -> Result<Option<Self>, String> {
        if !filepath.exists() {
            return Ok(None);
        }

        if !filepath.is_file() {
            return Err(format!("{} is not a file", filepath.display()));
        }

        let contents = fs::read_to_string(&filepath)
            .map_err(|e| e.to_string())?;

        Ok(Some(toml::from_str(&contents)
            .map_err(|e| e.to_string())?))
    }
}

fn default_data_file() -> PathBuf {
    PathBuf::from("place.hldr")
}

pub fn place(options: &Options, commit: bool) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(&options.data_file)?;
    let tokens = lex::lex(&content)?;
    let schemas = parse::parse(tokens)?;
    let validated = validate::validate(schemas)?;

    let mut client = load::new_client(&options.database_conn)?;
    let mut transaction = client.transaction()?;

    load::load(&mut transaction, &validated)?;

    if commit {
        println!("Committing changes");
        transaction.commit()?;
    } else {
        println!("Rolling back changes, pass `--commit` to apply")
    }

    Ok(())
}

#[cfg(test)]
mod root_tests {
    use std::env;

    use super::{place, Options, PathBuf};

    #[test]
    fn it_works() {
        let options = Options {
            data_file: PathBuf::from("test/fixtures/place.hldr".to_owned()),
            database_conn: env::var("HLDR_TEST_DATABASE_URL").unwrap().clone(),
        };

        place(&options, false).unwrap();
    }
}
