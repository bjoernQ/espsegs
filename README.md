# espsegs

## Installation

```
cargo install --git https://github.com/bjoernQ/espsegs
```

## Usage

```
Usage: espsegs [OPTIONS] --chip <CHIP> <FILE>

Arguments:
  <FILE>

Options:
  -c, --chip <CHIP>
  -s, --flash-size <SIZE>  [possible values: 256KB, 512KB, 1MB, 2MB, 4MB, 8MB, 16MB, 32MB, 64MB, 128MB, 256MB]
  -w, --width <WIDTH>      [default: 120]
  -h, --help               Print help (see more with '--help')
  -V, --version            Print version

```

### Example

```
❯ espsegs \projects\esp\esp-wifi\target\riscv32imc-unknown-none-elf\release\examples\embassy_dhcp --chip esp32c3

.text_dummy  3c000000  458784 DROM  [█████████████                                                                                                           ]
.rodata      3c070020   53060 DROM  [             █                                                                                                          ]
.rodata.wifi 3c07cf64   21756 DROM  [              ▏                                                                                                         ]

.rwdata_dumm 3fc80000   16948 DRAM  [██████                                                                                                                  ]
.data        3fc84238    4652 DRAM  [      █                                                                                                                 ]
.bss         3fc85468  121544 DRAM  [       ████████████████████████████████████████████                                                                     ]
.data.wifi   3fca2f30     360 DRAM  [                                                    ▏                                                                   ]

.trap        40380000    3188 IRAM  [    ▏                                                                                                                   ]
.rwtext      40380c74    2080 IRAM  [     ▏                                                                                                                  ]
.rwtext.wifi 40381494   11680 IRAM  [      ███                                                                                                               ]

.text_init   42000020     244 IROM  [▏                                                                                                                       ]
.text        42000114  402684 IROM  [███████████                                                                                                             ]
```
