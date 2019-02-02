// Copyright 2019, Sjors van Gelderen

use crate::attribute_table::AttributeTable;
use crate::pattern_table::PatternTable;
use crate::nametable::Nametable;
use crate::samples::Samples;

use std::fs::File;

use std::io::{
    Result,
    prelude::*
};

use std::path::Path;

pub fn load_pattern_table(path: &Path) -> Result<PatternTable> {
    let mut file = File::open(&path)?;
    let mut buffer: [u8; 8192] = [0u8; 8192]; // 8KB of graphics

    file.read(&mut buffer)?;

    let mut pixels: [u8; 32768] = [0u8; 32768]; // 512 tiles of 64 pixels

    // TODO: Consider how to read the data into pixels so that the pages are clearly separated

    for page_index in 0..2 {
        let page_start = page_index * 4096;
        let page_end = page_start + 4096;

        // The type of page is wrong
        // let page: &[u8] = match buffer.get(page_offset..page_limit) {
        //     Some(p) => p,
        //     _ => panic!("Failed to get page"),
        // };

        let page = &buffer[page_start..page_end];

        for (i, tile) in page.chunks(16).into_iter().enumerate() {
            let page_offset = page_index * 128;
            let tile_x_offset = i % 16 * 8;
            let tile_y_offset = (i as f32 / 16.0).floor() as usize * 256 * 8;

            for y in 0..8 {
                for x in 0..8 {
                    let test = 0b10000000 >> x;
                    let lower = tile[y] & test;
                    let higher = tile[y + 8] & test;

                    // Different layout:
                    // pixels[tile_index * 64 + y * 8 + x]

                    pixels[page_offset + tile_y_offset + tile_x_offset + y * 256 + x] =
                        match (lower > 0u8, higher > 0u8) {
                            (true, true) => 3u8,
                            (false, true) => 2u8,
                            (true, false) => 1u8,
                            (false, false) => 0u8
                        };
                }
            }
        }
    }
    

    // for y in 0..8 {
    //     for x in 0..8 {
    //         let b = pixels[y * 8 + x];
    //         print!("{:?}", b);
    //     }
    //     println!();
    // }

    let pattern: PatternTable = PatternTable {
        bytes: buffer,
        pixels: pixels,
    };

    Ok(pattern)
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
