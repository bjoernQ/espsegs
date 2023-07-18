use std::{error::Error, fs, path::PathBuf, process::exit};

use clap::Parser;
use object::{Object, ObjectSection};

const WIDTH: usize = 120;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file: PathBuf,
    #[arg(short, long)]
    chip: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let bin_data = fs::read(args.file)?;
    let obj_file = object::File::parse(&*bin_data)?;
    let sections = obj_file.sections();

    let mut sections: Vec<object::Section> = sections
        .into_iter()
        .filter(|section| section.address() != 0 && section.size() != 0)
        .collect();
    sections.sort_by(|a, b| a.address().partial_cmp(&b.address()).unwrap());

    let chip = args.chip.replace("-", "").to_ascii_lowercase();
    let chip_memory = MEMORY
        .iter()
        .find(|m| m.name.replace("-", "").to_ascii_lowercase() == chip);

    if let None = chip_memory {
        println!("Unknown chip");
        exit(1);
    }

    let chip_memory = chip_memory.unwrap();

    let mut last_region = usize::MAX;

    for section in sections {
        let region = chip_memory.regions.iter().find(|region| {
            region.start <= section.address() && region.end >= (section.address() + section.size())
        });

        if let Some(ref region) = &region {
            if region.id != last_region {
                println!();
                last_region = region.id;
            }
        }

        print!(
            "{:<12.12} {:8x} {:7}",
            section.name().unwrap(),
            section.address(),
            section.size()
        );

        if let Some(ref region) = &region {
            print!(" {:5} ", region.name);
            print_memory(region.start, region.end, section.address(), section.size());
        }

        println!();
    }

    Ok(())
}

fn print_memory(region_start: u64, region_end: u64, block_start: u64, block_size: u64) {
    let region_size = region_end - region_start;
    let offset =
        ((WIDTH as f64 / region_size as f64) * (block_start as f64 - region_start as f64)) as u32;
    let w = ((WIDTH as f64 / region_size as f64) * block_size as f64) as u32;

    let (small, w) = if w == 0 { (true, 1) } else { (false, w) };

    print!("[");

    for _ in 0..offset {
        print!(" ");
    }
    for _ in 0..w {
        if small {
            print!("\u{258f}");
        } else {
            print!("\u{2588}");
        }
    }
    for _ in 0..(WIDTH as u32 - w - offset) {
        print!(" ");
    }
    print!("]");
}

pub struct Memory {
    name: &'static str,
    regions: &'static [MemoryRegion],
}

pub struct MemoryRegion {
    id: usize,
    name: &'static str,
    start: u64,
    end: u64,
}

// TODO double check and add more chips
const MEMORY: &[Memory] = &[
    Memory {
        name: "ESP32",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x3FFB0000,
                end: 0x3FFB0000 + 176 * 1024,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x40080000,
                end: 0x40080000 + 128 * 1024,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3F400000,
                end: 0x3F400000 + 4 * 1024 * 1024,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x400D0000,
                end: 0x400D0000 + 4 * 1024 * 1024,
            },
        ],
    },
    Memory {
        name: "ESP32-S3",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x3FC8_8000,
                end: 0x3FCE_FFFF,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x4037_8000,
                end: 0x403D_FFFF,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3C00_0000,
                end: 0x3DFF_FFFF,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x4200_0000,
                end: 0x43FF_FFFF,
            },
        ],
    },
    Memory {
        name: "ESP32-C3",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x3FC80000,
                end: 0x3FC80000 + 0x50000,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x4037C000,
                end: 0x4037C000 + 400 * 1024,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3C000000,
                end: 0x3C000000 + 0x400000,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x42000000,
                end: 0x42000000 + 0x400000,
            },
        ],
    },
];
