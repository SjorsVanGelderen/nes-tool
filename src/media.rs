// Copyright 2019, Sjors van Gelderen

use crate::attribute_table::AttributeTable;
use crate::character::Character;
use crate::nametable::Nametable;
use crate::samples::Samples;

use std::fs::File;

use std::io::{
    Result,
    prelude::*
};

use std::path::Path;

pub fn load_character(path: &Path) -> Result<Character> {
    let mut file = File::open(&path)?;
    let mut buffer: [u8; 8192] = [0u8; 8192]; // 8KB of graphics

    file.read(&mut buffer)?;

    let mut character: [u8; 32768] = [0u8; 32768]; // 512 tiles of 64 pixels

    for (tile_index, tile) in buffer.chunks(16).into_iter().enumerate() {
        for y in 0..8 {
            for x in 0..8 {
                let test = 0b10000000 >> x;
                let lower = tile[y] & test;
                let higher = tile[y + 8] & test;

                character[tile_index * 64 + y * 8 + x] =
                    match (lower > 0u8, higher > 0u8) {
                        (true, true) => 3u8,
                        (false, true) => 2u8,
                        (true, false) => 1u8,
                        (false, false) => 0u8
                    };
            }
        }
    }

    for y in 0..8 {
        for x in 0..8 {
            let b = character[y * 8 + x];
            print!("{:?}", b);
        }
        println!();
    }

    // TODO: Convert the graphics data into a GLSL sampler
    // where every texel is a GL_RED value

    Ok(Character::zero())
}

pub fn load_samples(path: &Path) -> Result<Samples> {
    let mut file = File::open(&path)?;
    let mut samples: [u8; 26] = [0u8; 26];

    file.read(&mut samples)?;

    Ok(Samples::zero())
}

pub fn load_nametable(path: &Path) -> Result<Nametable> {
    let mut file = File::open(&path)?;
    let mut nametable: [u8; 1024] = [0u8; 1024];

    file.read(&mut nametable)?;

    Ok(Nametable::zero())
}

pub fn load_attribute_table(path: &Path) -> Result<AttributeTable> {
    let mut file = File::open(&path)?;
    let mut attribute_table: [u8; 64] = [0u8; 64];

    file.read(&mut attribute_table)?;
    
    Ok(AttributeTable::zero())
}

pub fn save_character(path: &str) -> Result<()> {
    // let mut file = File::create(path)?;

    // file.write_all(b"Hello, World!")?;

    Ok(())
}

pub fn save_samples() -> Result<()> {
    Ok(())
}

pub fn save_nametable() -> Result<()> {
    Ok(())
}

pub fn save_attributes() -> Result<()> {
    Ok(())
}
