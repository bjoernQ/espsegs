use std::{error::Error, fs, path::PathBuf, process::exit};

use clap::{Parser, ValueEnum};
use object::{Object, ObjectSection};

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FlashSize {
    /// 256 KB
    _256Kb,
    /// 512 KB
    _512Kb,
    /// 1 MB
    _1Mb,
    /// 2 MB
    _2Mb,
    /// 4 MB
    _4Mb,
    /// 8 MB
    _8Mb,
    /// 16 MB
    _16Mb,
    /// 32 MB
    _32Mb,
    /// 64 MB
    _64Mb,
    /// 128 MB
    _128Mb,
    /// 256 MB
    _256Mb,
}

impl FlashSize {
    fn bytes(self) -> u64 {
        match self {
            FlashSize::_256Kb => 256 * 1024,
            FlashSize::_512Kb => 512 * 1024,
            FlashSize::_1Mb => 1024 * 1024,
            FlashSize::_2Mb => 2 * 1024 * 1024,
            FlashSize::_4Mb => 4 * 1024 * 1024,
            FlashSize::_8Mb => 8 * 1024 * 1024,
            FlashSize::_16Mb => 16 * 1024 * 1024,
            FlashSize::_32Mb => 32 * 1024 * 1024,
            FlashSize::_64Mb => 64 * 1024 * 1024,
            FlashSize::_128Mb => 128 * 1024 * 1024,
            FlashSize::_256Mb => 256 * 1024 * 1024,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file: PathBuf,

    #[arg(short, long)]
    chip: String,

    #[arg(short = 's', long, value_name = "SIZE", value_enum)]
    flash_size: Option<FlashSize>,

    #[arg(short = 'w', long, default_value = "120")]
    width: usize,
}

fn normalize(chip_name: &str) -> String {
    chip_name.replace('-', "").to_ascii_lowercase()
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

    let chip = normalize(&args.chip);
    let chip_memory = MEMORY.iter().find(|m| normalize(m.name) == chip);

    let Some(chip_memory) = chip_memory else {
        println!("Unknown chip");
        exit(1);
    };

    let mut last_region = usize::MAX;

    // Calculate max section name width for the first column
    let mut section_name_max_width = 0;
    for section in sections.iter() {
        let name = section.name().unwrap();
        if name.len() > section_name_max_width {
            section_name_max_width = name.len();
        }
    }

    for section in sections {
        let region = chip_memory.regions.iter().find(|region| {
            region.start <= section.address()
                && region.end(args.flash_size) >= (section.address() + section.size())
        });

        if let Some(region) = &region {
            if region.id != last_region {
                println!();
                last_region = region.id;
            }
        }

        print!(
            "{:width$} {:8x} {:7}",
            section.name().unwrap(),
            section.address(),
            section.size(),
            width = section_name_max_width,
        );

        if let Some(region) = &region {
            print!(" {:8} ", region.name);
            print_memory(
                region.start,
                region.end(args.flash_size),
                section.address(),
                section.size(),
                args.width - section_name_max_width - 26, // 26 = `address` + `size` + spaces + brackets + region name
            );
        }

        println!();
    }

    Ok(())
}

fn print_memory(
    region_start: u64,
    region_end: u64,
    block_start: u64,
    block_size: u64,
    width: usize,
) {
    let region_size = region_end - region_start;
    let offset =
        ((width as f64 / region_size as f64) * (block_start as f64 - region_start as f64)) as usize;
    let w = ((width as f64 / region_size as f64) * block_size as f64) as usize;

    let small = w == 0;
    let w = w.max(1);

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
    for _ in 0..(width - w - offset) {
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
    length: u64,
}

impl MemoryRegion {
    pub fn end(&self, flash_size: Option<FlashSize>) -> u64 {
        let length = match self.name.ends_with("ROM") && flash_size.is_some() {
            true => flash_size.unwrap().bytes(),
            false => self.length,
        };

        self.start + length
    }
}

#[cfg(not(feature = "expert"))]
const MEMORY: &[Memory] = &[
    Memory {
        name: "ESP32",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x3FFB0000,
                length: 176 * 1024,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x40080000,
                length: 128 * 1024,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3F400000,
                length: 4 * 1024 * 1024,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x400D0000,
                length: 4 * 1024 * 1024,
            },
        ],
    },
    Memory {
        name: "ESP32-S2",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x3FFB0000,
                length: 0x40000000 - 0x3FFB0000,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x40020000,
                length: 0x40070000 - 0x40020000,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3F000000,
                length: 0x3FF80000 - 0x3F000000,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x40080000,
                length: 0x40800000 - 0x40080000,
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
                length: 0x3FCE_FFFF - 0x3FC8_8000,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x4037_8000,
                length: 0x403D_FFFF - 0x4037_8000,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3C00_0000,
                length: 0x3DFF_FFFF - 0x3C00_0000,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x4200_0000,
                length: 0x43FF_FFFF - 0x4200_0000,
            },
        ],
    },
    Memory {
        name: "ESP32-C2",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x3FCA0000,
                length: 0x3FCE0000 - 0x3FCA0000,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x4037C000,
                length: 0x403C0000 - 0x4037C000,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3C000000,
                length: 0x3C400000 - 0x3C000000,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x42000000,
                length: 0x42400000 - 0x42000000,
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
                length: 0x50000,
            },
            MemoryRegion {
                id: 1,
                name: "IRAM",
                start: 0x4037C000,
                length: 400 * 1024,
            },
            MemoryRegion {
                id: 2,
                name: "DROM",
                start: 0x3C000000,
                length: 0x400000,
            },
            MemoryRegion {
                id: 3,
                name: "IROM",
                start: 0x42000000,
                length: 0x400000,
            },
        ],
    },
    Memory {
        name: "ESP32-C6",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "RAM",
                start: 0x40800000,
                length: 0x40880000 - 0x40800000,
            },
            MemoryRegion {
                id: 1,
                name: "ROM",
                start: 0x42000000,
                length: 0x10000 << 8,
            },
        ],
    },
    Memory {
        name: "ESP32-H2",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x40800000,
                length: 0x40850000 - 0x40800000,
            },
            MemoryRegion {
                id: 1,
                name: "ROM",
                start: 0x42000000,
                length: 0x10000 << 8,
            },
        ],
    },
];

#[cfg(feature = "expert")]
const MEMORY: &[Memory] = &[
    Memory {
        name: "ESP32",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DROM",
                start: 0x3F400000,
                length: 4 * 1024 * 1024,
            },
            MemoryRegion {
                id: 1,
                name: "IROM",
                start: 0x400D0000,
                length: 4 * 1024 * 1024,
            },
            MemoryRegion {
                id: 2,
                name: "DRTC_FAST",
                start: 0x3FF8_0000,
                length: 0x3FF8_1FFF - 0x3FF8_0000,
            },
            MemoryRegion {
                id: 3,
                name: "DRAM2",
                start: 0x3FFA_E000,
                length: 0x3FFD_FFFF - 0x3FFA_E000,
            },
            MemoryRegion {
                id: 4,
                name: "DRAM1",
                start: 0x3FFE_0000,
                length: 0x3FFF_FFFF - 0x3FFE_0000,
            },
            MemoryRegion {
                id: 5,
                name: "Cache",
                start: 0x4007_0000,
                length: 0x4007_FFFF - 0x4007_0000,
            },
            MemoryRegion {
                id: 6,
                name: "IRAM0",
                start: 0x4008_0000,
                length: 0x4009_FFFF - 0x4008_0000,
            },
            MemoryRegion {
                id: 7,
                name: "IRAM1",
                start: 0x400A_0000,
                length: 0x400A_FFFF - 0x400A_0000,
            },
            MemoryRegion {
                id: 8,
                name: "IRAM1*",
                start: 0x400B_8000,
                length: 0x400B_FFFF - 0x400B_8000,
            },
            MemoryRegion {
                id: 9,
                name: "IRTC_FAST",
                start: 0x400C_0000,
                length: 0x400C_1FFF - 0x400C_0000,
            },
            MemoryRegion {
                id: 10,
                name: "RTC_SLOW",
                start: 0x5000_0000,
                length: 0x5000_1FFF - 0x5000_0000,
            },
        ],
    },
    Memory {
        name: "ESP32-S2",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DROM",
                start: 0x3F000000,
                length: 0x3FF80000 - 0x3F000000,
            },
            MemoryRegion {
                id: 1,
                name: "IROM",
                start: 0x40080000,
                length: 0x40800000 - 0x40080000,
            },
            MemoryRegion {
                id: 2,
                name: "DRTC_FAST",
                start: 0x3FF9_E000,
                length: 0x3FF9_FFFF - 0x3FF9_E000,
            },
            MemoryRegion {
                id: 3,
                name: "DRAM0",
                start: 0x3FFB_0000,
                length: 0x3FFB_7FFF - 0x3FFB_0000,
            },
            MemoryRegion {
                id: 4,
                name: "DRAM1",
                start: 0x3FFB_8000,
                length: 0x3FFF_FFFF - 0x3FFB_8000,
            },
            MemoryRegion {
                id: 5,
                name: "IRAM0",
                start: 0x4002_0000,
                length: 0x4002_7FFF - 0x4002_0000,
            },
            MemoryRegion {
                id: 6,
                name: "IRAM1",
                start: 0x4002_8000,
                length: 0x4006_FFFF - 0x4002_8000,
            },
            MemoryRegion {
                id: 7,
                name: "IRTC_FAST",
                start: 0x4007_0000,
                length: 0x4007_1FFF - 0x4007_0000,
            },
            MemoryRegion {
                id: 8,
                name: "RTC_SLOW",
                start: 0x5000_0000,
                length: 0x5000_1FFF - 0x5000_0000,
            },
        ],
    },
    Memory {
        name: "ESP32-S3",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DROM",
                start: 0x3C00_0000,
                length: 0x3DFF_FFFF - 0x3C00_0000,
            },
            MemoryRegion {
                id: 1,
                name: "IROM",
                start: 0x4200_0000,
                length: 0x43FF_FFFF - 0x4200_0000,
            },
            MemoryRegion {
                id: 2,
                name: "DRAM1",
                start: 0x3FC8_8000,
                length: 0x3FCE_FFFF - 0x3FC8_8000,
            },
            MemoryRegion {
                id: 3,
                name: "DRAM2",
                start: 0x3FCF_0000,
                length: 0x3FCF_FFFF - 0x3FCF_0000,
            },
            MemoryRegion {
                id: 4,
                name: "IRAM1",
                start: 0x4037_0000,
                length: 0x4037_7FFF - 0x4037_0000,
            },
            MemoryRegion {
                id: 5,
                name: "IRAM2",
                start: 0x4037_8000,
                length: 0x403D_FFFF - 0x4037_8000,
            },
            MemoryRegion {
                id: 6,
                name: "RTC_SLOW",
                start: 0x5000_0000,
                length: 0x5000_1FFF - 0x5000_0000,
            },
            MemoryRegion {
                id: 7,
                name: "RTC_FAST",
                start: 0x600F_E000,
                length: 0x600F_FFFF - 0x600F_E000,
            },
        ],
    },
    Memory {
        name: "ESP32-C2",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DROM",
                start: 0x3C000000,
                length: 0x3C400000 - 0x3C000000,
            },
            MemoryRegion {
                id: 1,
                name: "IROM",
                start: 0x42000000,
                length: 0x42400000 - 0x42000000,
            },
            MemoryRegion {
                id: 2,
                name: "DRAM1",
                start: 0x3FCA_0000,
                length: 0x3FCD_FFFF - 0x3FCA_0000,
            },
            MemoryRegion {
                id: 3,
                name: "IRAM0",
                start: 0x4037_C000,
                length: 0x4037_FFFF - 0x4037_C000,
            },
            MemoryRegion {
                id: 4,
                name: "IRAM1",
                start: 0x4038_0000,
                length: 0x403B_FFFF - 0x4038_0000,
            },
        ],
    },
    Memory {
        name: "ESP32-C3",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DROM",
                start: 0x3C000000,
                length: 0x400000,
            },
            MemoryRegion {
                id: 1,
                name: "IROM",
                start: 0x42000000,
                length: 0x400000,
            },
            MemoryRegion {
                id: 2,
                name: "DRAM1",
                start: 0x3FC8_0000,
                length: 0x3FCD_FFFF - 0x3FC8_0000,
            },
            MemoryRegion {
                id: 3,
                name: "IRAM0",
                start: 0x4037_C000,
                length: 0x4037_FFFF - 0x4037_C000,
            },
            MemoryRegion {
                id: 4,
                name: "IRAM1",
                start: 0x4038_0000,
                length: 0x403D_FFFF - 0x4038_0000,
            },
            MemoryRegion {
                id: 4,
                name: "RTCFAST",
                start: 0x5000_0000,
                length: 0x5000_1FFF - 0x5000_0000,
            },
        ],
    },
    Memory {
        name: "ESP32-C6",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "RAM",
                start: 0x40800000,
                length: 0x40880000 - 0x40800000,
            },
            MemoryRegion {
                id: 1,
                name: "ROM",
                start: 0x42000000,
                length: 0x10000 << 8,
            },
            MemoryRegion {
                id: 2,
                name: "LPRAM",
                start: 0x5000_0000,
                length: 0x5000_3FFF - 0x5000_0000,
            },
        ],
    },
    Memory {
        name: "ESP32-H2",
        regions: &[
            MemoryRegion {
                id: 0,
                name: "DRAM",
                start: 0x40800000,
                length: 0x40850000 - 0x40800000,
            },
            MemoryRegion {
                id: 1,
                name: "ROM",
                start: 0x42000000,
                length: 0x10000 << 8,
            },
            MemoryRegion {
                id: 2,
                name: "LPRAM",
                start: 0x5000_0000,
                length: 0x5000_0FFF - 0x5000_0000,
            },
        ],
    },
];
