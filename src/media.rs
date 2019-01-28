// Copyright 2019, Sjors van Gelderen

use std::fs::File;

use std::io::{
    Result,
    prelude::*
};

fn load_charachter(path: &str) -> Result<()> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;
    assert_eq!(contents, "Hello, World!");

    Ok(())
}

fn load_palettes() -> Result<()> {
    Ok(())
}

fn load_nametable() -> Result<()> {
    Ok(())
}

fn load_attributes() -> Result<()> {
    Ok(())
}

fn save_character(path: &str) -> Result<()> {
    let mut file = File::create(path)?;

    file.write_all(b"Hello, World!")?;

    Ok(())
}

fn save_palettes() -> Result<()> {
    Ok(())
}

fn save_nametable() -> Result<()> {
    Ok(())
}

fn save_attributes() -> Result<()> {
    Ok(())
}
