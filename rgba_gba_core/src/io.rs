// io.rs --- 
// 
// Filename: io.rs
// Author: Louise <louise>
// Created: Wed Jan  3 15:30:01 2018 (+0100)
// Last-Updated: Thu Jan 18 20:53:50 2018 (+0100)
//           By: Louise <louise>
//
use gpu::GPU;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::File;
use std::io::Read;

pub struct Interconnect {
    bios: Vec<u8>,
    iram: [u8; 0x8000],

    gpu: GPU,

    cycles_to_spend: u32,
    waitstates: [[[u32; 2]; 3]; 16],

    postflg: u8,
    ime: bool,
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect {
            bios: vec![],
            iram: [0; 0x8000],
            gpu: GPU::new(),

            cycles_to_spend: 0,
            waitstates: [
                [[1, 1], [1, 1], [1, 1]], // BIOS
                [[1, 1], [1, 1], [1, 1]],
                [[3, 3], [3, 3], [6, 6]], // ERAM
                [[1, 1], [1, 1], [1, 1]], // IRAM
                [[1, 1], [1, 1], [1, 1]], // IO
                [[1, 1], [1, 1], [2, 2]], // Palette RAM
                [[1, 1], [1, 1], [2, 2]], // VRAM
                [[1, 1], [1, 1], [1, 1]], // OAM
                [[5, 5], [5, 5], [8, 8]], // GamePak WaitState 0
                [[5, 5], [5, 5], [8, 8]],
                [[5, 5], [5, 5], [8, 8]], // GamePak WaitState 1
                [[5, 5], [5, 5], [8, 8]],
                [[5, 5], [5, 5], [8, 8]], // GamePak WaitState 2
                [[5, 5], [5, 5], [8, 8]],
                [[1, 1], [1, 1], [1, 1]], // GamePak SRAM
                [[1, 1], [1, 1], [1, 1]],
            ],

            postflg: 0,
            ime: false,
        }
    }

    pub fn declare_access(&mut self, address: usize, width: usize) {
        self.cycles_to_spend += self.waitstates[address >> 24][width][0];
    }

    pub fn spend(&mut self) {
        self.gpu.spend_cycles(self.cycles_to_spend);
        
        self.cycles_to_spend = 0;
    }
    
    pub fn read_u32(&self, address: usize) -> u32 {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 =>
                LittleEndian::read_u32(&self.bios[address..]),
            0x03000000 =>
                LittleEndian::read_u32(&self.iram[(address & 0x7fff)..]),
            _ => unimplemented!(),
        }
    }

    pub fn read_u16(&self, address: usize) -> u16 {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 =>
                LittleEndian::read_u16(&self.bios[address..]),
            0x03000000 =>
                LittleEndian::read_u16(&self.iram[(address & 0x7fff)..]),
            _ => unimplemented!(),
        }
    }

    pub fn read_u8(&self, address: usize) -> u8 {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => self.bios[address],
            0x03000000 => self.iram[address & 0x7fff],
            0x04000000 => self.io_read_u8(address),
            _ => unimplemented!("Reading a byte from {:08x}", address)
        }
    }

    fn io_read_u8(&self, address: usize) -> u8 {
        match address {
            IME => self.ime as u8,
            POSTFLG => self.postflg,
            _ => unimplemented!(),
        }
    }

    pub fn write_u32(&mut self, address: usize, value: u32) {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => warn!("Ignored write to BIOS"),
            0x03000000 => LittleEndian::write_u32(
                &mut self.iram[(address & 0x7fff)..], value
            ),
            _ => unimplemented!("Writing 4 bytes to {:08x}", address),
        }
    }

    pub fn write_u16(&mut self, address: usize, value: u16) {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => warn!("Ignored write to BIOS"),
            0x03000000 => LittleEndian::write_u16(
                &mut self.iram[(address & 0x7fff)..], value
            ),
            0x04000000 => self.io_write_u16(address, value),
            _ => unimplemented!("Writing 2 bytes to {:08x}", address),
        }
    }
    
    pub fn write_u8(&mut self, address: usize, value: u8) {
        match address & 0x0F000000 {
            0x00000000 if address < 0x4000 => warn!("Ignored write to BIOS"),
            0x03000000 => self.iram[address & 0x7fff] = value,
            0x04000000 => self.io_write_u8(address, value),
            _ => unimplemented!("Writing a byte to {:08x}", address),
        }
    }

    fn io_write_u16(&mut self, address: usize, value: u16) {
        match address {
            0x04000000...0x04000056 => self.gpu.io_write_u16(address, value),
            _ => unimplemented!(),
        }
    }
    
    fn io_write_u8(&mut self, address: usize, value: u8) {
        match address {
            IME => self.ime = value != 0,
            _ => unimplemented!(),
        }
    }
    
    pub fn load_bios(&mut self, filename: &str) -> Result<(), &'static str> {
        match File::open(filename) {
            Ok(mut file) => {    
                debug!("BIOS file openend");
            
                if let Err(e) = file.read_to_end(&mut self.bios) {
                    warn!("Error reading BIOS file : {}", e);
                    Err("Error reading BIOS")
                } else {
                    Ok(())
                }
            }

            Err(e) => {
                warn!("Couldn't load BIOS : {}", e);
                Err("Error opening BIOS file")
            }
        }
    }
}

const POSTFLG: usize = 0x04000300;
const IME: usize = 0x04000208;
