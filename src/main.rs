/*
 * Copyright (C) 2018 Aron Heinecke
 *
 * This file is part of restore_revert.
 * 
 * restore_revert is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * 
 * restore_revert is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License
 * along with restore_revert.  If not, see <http://www.gnu.org/licenses/>.
 */

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate clap;

use clap::{Arg,App};

use std::path::{Path,PathBuf};
use std::vec::Vec;
use regex::Regex;
use std::fs::{rename,remove_file};

lazy_static! {
// according to https://github.com/bit-team/backintime/wiki/FAQ#after-restore-i-have-duplicates-with-extension-backup-20131121
    static ref REG: Regex = Regex::new("(.*)\\.backup\\.[[:digit:]]{8}").unwrap();
}

struct Reverted {
    backup: PathBuf,
    reverted: PathBuf
}

/// backintime restore-reverter by Aron Heinecke
/// Allows to undo a restore with backup option enabled, by deleting the restored file and
/// renaming the .backup.{date} original file.
/// It is not capable of reverting restored files that were not existing originally
fn main() {
    let matches = App::new("restore_revert")
        .version("1.0")
        .author("Aron H. <aron.heinecke@t-online.de>")
        .about("Revert backintime restore of files")
        .arg(Arg::with_name("dir")
            .short("d")
            .long("dir")
            .takes_value(true)
            .value_name("PATH")
            .required(true)
            .help("start directory for revert"))
        .arg(Arg::with_name("simulate")
            .short("s")
            .long("simulate")
            .takes_value(false)
            .help("simulate revert, do not change files"))
        .arg(Arg::with_name("rename-symlink")
            .short("rs")
            .long("rename-symlink")
            .takes_value(false)
            .help("rename symlinks"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .help("verbose output"))
        .arg(Arg::with_name("followsymlink")
            .takes_value(false)
            .long("follow-symlink")
            .help("Follow symlinks. Warning: use with caution, experimental!"))
        .get_matches();
    
    let dir = matches.value_of("dir").unwrap();
    let rename_symlink = matches.is_present("rename-symlink");
    let follow_symlink = matches.is_present("followsymlink");
    let simulate = matches.is_present("simulate");
    let verbose = matches.is_present("verbose");
    
    let path = PathBuf::from(dir);
    let mut list: Vec<Reverted> = Vec::new();
    let mut seen: i64 = 0;
    if path.is_dir() {
        println!("Using path {:?}",dir);
        if simulate {
            println!("Simulating");
        }
        find_files(&path, &mut list,&mut seen, &rename_symlink, &follow_symlink,&verbose);
        
        let size = list.len();
        for rev in list {
            println!("Found file {:?} origin {:?}",rev.backup,rev.reverted);
            if !simulate {
                let name = rev.reverted.file_name().unwrap().to_str().unwrap();
                let mut rev_rename = rev.reverted.parent().expect("No Parent!").to_path_buf();
                rev_rename.push(format!("{}_reverted",name));
                println!("Renaming to {:?}",rev_rename);
                rename(&rev.reverted,&rev_rename)
                    .expect(&format!("Unable to rename {:?}",rev.reverted));
                rename(&rev.backup,&rev.reverted)
                    .expect(&format!("Unable to rename {:?}",rev.backup));
                remove_file(&rev_rename).expect(&format!("Unable to delete {:?}",rev_rename));
            }
        }
        println!("Finished, restore-reverted {} of {} files",size,seen);
    } else {
        eprintln!("ERROR: Invalid path {:?}",path);
    }
}

/// Traverse folders and check files
fn find_files(folder: &Path,mut list: &mut Vec<Reverted>, mut seen: &mut i64,
        rename_symlink: &bool, follow_symlink: &bool, verbose: &bool) {
    for entry in folder.read_dir().expect(&format!("unable to read dir {:?}",folder)) {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                let path = entry.path();
                let is_symlink = metadata.file_type().is_symlink();
                if metadata.is_dir() && (!is_symlink || *follow_symlink) {
                    find_files(&path, &mut list, &mut seen, rename_symlink,follow_symlink,verbose);
                    if is_symlink && *rename_symlink {
                        *seen += 1;
                        check_file(path,&mut list);
                    }
                } else if !is_symlink || *rename_symlink {
                    *seen += 1;
                    check_file(path,&mut list);
                } else if *verbose {
                    println!("Ignoring {:?}",path);
                }
            } else {
                eprintln!("ERROR: Unable to read metadata for {:?}",entry.path());
                panic!("");
            }
            
        }
    }
}

/// Check file and add if existing
fn check_file(file: PathBuf, list: &mut Vec<Reverted>) {
    let name = file.file_name().expect(&format!("No filename for {:?}",file));
    
    if let Some(capture) = REG.captures(name.to_str()
            .expect(&format!("Invalid utf8 found in {:?}",file))) {
        let mut original = file.clone();
        original.set_file_name(capture.get(1).unwrap().as_str());
        
        if let Ok(morigin) = original.metadata() {
            if morigin.is_file() {
                list.push(
                    Reverted {
                        backup: file.clone(),
                        reverted: original
                    }
                );
            } else {
                println!("WARN: found match but no reverted file {:?}",file);
            }
        } else {
            println!("WARN: no metadata for {:?}",file);
        }
    }
}
