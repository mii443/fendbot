use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{anyhow, Result};

pub fn save_context(ctx: &fend_core::Context, id: u64) {
    let file = File::create(format!("./context/{}", id)).unwrap();
    let mut writer = BufWriter::new(file);
    ctx.serialize_variables(&mut writer).unwrap();
}

pub fn read_context(path: PathBuf) -> Result<fend_core::Context> {
    let mut context = fend_core::Context::new();
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    if let Err(_) = context.deserialize_variables(&mut reader) {
        return Err(anyhow!("Cannot deserialize variables"));
    }

    Ok(context)
}

pub fn restore_contexts() -> HashMap<u64, fend_core::Context> {
    let mut result = HashMap::new();
    let mut dir = if let Ok(dir) = std::fs::read_dir("./context") {
        dir
    } else {
        std::fs::create_dir("./context").unwrap();
        return HashMap::new();
    };

    while let Some(Ok(entry)) = dir.next() {
        let id = if let Ok(id) = u64::from_str_radix(entry.file_name().to_str().unwrap(), 10) {
            id
        } else {
            continue;
        };

        if let Ok(context) = read_context(entry.path()) {
            result.insert(id, context);
        }
    }

    result
}
