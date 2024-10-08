use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use tracing::{error, trace};

pub fn create_context() -> fend_core::Context {
    trace!("Creating new context");
    let mut context = fend_core::Context::new();
    context.set_random_u32_fn(rand::random);
    context.define_custom_unit_v1(
        "item",
        "items",
        "!",
        &fend_core::CustomUnitAttribute::AllowLongPrefix,
    );
    context.define_custom_unit_v1(
        "I",
        "items",
        "item",
        &fend_core::CustomUnitAttribute::AllowShortPrefix,
    );
    context.define_custom_unit_v1(
        "stack",
        "",
        "64",
        &fend_core::CustomUnitAttribute::IsLongPrefix,
    );
    context.define_custom_unit_v1(
        "chest",
        "",
        "27 stack",
        &fend_core::CustomUnitAttribute::IsLongPrefix,
    );
    context.define_custom_unit_v1(
        "largechest",
        "",
        "2 chest",
        &fend_core::CustomUnitAttribute::IsLongPrefix,
    );
    context
}

pub fn save_context(ctx: &fend_core::Context, id: u64) {
    trace!("Saving context");
    let file = File::create(format!("./context/{}", id)).unwrap();
    let mut writer = BufWriter::new(file);
    ctx.serialize_variables(&mut writer).unwrap();
}

pub fn read_context(path: PathBuf) -> Result<fend_core::Context> {
    trace!("Reading context");
    let mut context = create_context();

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    if let Err(_) = context.deserialize_variables(&mut reader) {
        error!("Cannot deserialize variable");
        return Err(anyhow!("Cannot deserialize variables"));
    }

    Ok(context)
}

pub fn restore_contexts() -> HashMap<u64, fend_core::Context> {
    trace!("Restoring contexts");
    let mut result = HashMap::new();
    let mut dir = if let Ok(dir) = std::fs::read_dir("./context") {
        dir
    } else {
        trace!("Creating context dir");
        std::fs::create_dir("./context").unwrap();
        return HashMap::new();
    };

    while let Some(Ok(entry)) = dir.next() {
        let id = if let Ok(id) = u64::from_str_radix(entry.file_name().to_str().unwrap(), 10) {
            id
        } else {
            trace!("Cannot parse id: {}", entry.file_name().to_str().unwrap());
            continue;
        };

        if let Ok(context) = read_context(entry.path()) {
            result.insert(id, context);
        }
    }

    result
}
