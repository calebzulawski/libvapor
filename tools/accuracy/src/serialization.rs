//! Fixture format, loading and writing.

use std::{fs, io::Write, path::Path, sync::OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Width {
    F32,
    F64,
}

impl Width {
    pub fn value(self, bits: u64) -> f64 {
        match self {
            Self::F32 => f32::from_bits(bits.try_into().unwrap()) as f64,
            Self::F64 => f64::from_bits(bits),
        }
    }

    pub fn bits(self, value: f64) -> u64 {
        match self {
            Self::F32 => (value as f32).to_bits() as u64,
            Self::F64 => value.to_bits(),
        }
    }
}

// Raw bits preserve NaNs, infinities and signed zeros in JSON.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Case {
    pub input: [u64; 3],
    // One [lower, upper] pair per output; sincos has two outputs.
    pub bounds: Vec<[u64; 2]>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Fixtures {
    pub width: Width,
    pub op: String,
    pub cases: Vec<Case>,
}

pub const FIXTURE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data/mpfr.json");

pub fn load(path: impl AsRef<Path>) -> Result<Vec<Fixtures>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

pub fn cases(width: Width, op: &str) -> &'static [Case] {
    static DATA: OnceLock<Vec<Fixtures>> = OnceLock::new();
    &DATA
        .get_or_init(|| load(FIXTURE_PATH).expect("cannot load accuracy fixtures"))
        .iter()
        .find(|group| group.width == width && group.op == op)
        .unwrap_or_else(|| panic!("missing {width:?} {op} fixtures"))
        .cases
}

pub fn save(path: impl AsRef<Path>, groups: &[Fixtures]) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;
    serde_json::to_writer(&mut file, groups)?;
    writeln!(file)?;
    Ok(())
}
