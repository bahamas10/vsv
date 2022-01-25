/*
 * A rust replacement for vsv
 *
 * Original: https://github.com/bahamas10/vsv
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

use std::fs;
use std::path;

use anyhow::{anyhow, Result};

#[derive(Debug)]
enum ServiceState {
    Run,
    Down,
    Finish
}

static BASE_DIR: &str = "./test/service";

fn main() {
    let services = match get_services() {
        Ok(svcs) => svcs,
        Err(err) => panic!("failed to get services: {:?}", err),
    };

    println!("services = {:?}", services);

    for service in services {
        let _ = process_service(service);
    }
}

fn process_service(dir: path::PathBuf) -> Result<()> {
    let name = match dir.file_name() {
        Some(name) => name,
        None => return Err(anyhow!("failed to get name from service")),
    };

    println!("dir = {:?}", dir);
    println!("name = {:?}", name);

    let pid = match get_pid(&dir) {
        Ok(pid) => pid,
        Err(err) => panic!("failed to get pid: {:?}", err),
    };

    println!("pid = {:?}", pid);

    let state = match get_state(&dir) {
        Ok(state) => state,
        Err(err) => panic!("failed to get state: {:?}", err),
    };

    println!("state = {:?}", state);

    Ok(())
}

fn get_services() -> Result<Vec<path::PathBuf>> {
    // loop services directory and collect service names
    let mut dirs = Vec::new();

    for entry in fs::read_dir(BASE_DIR)? {
        let entry = entry?;
        let path = entry.path();

        if ! path.is_dir() {
            continue;
        }

        dirs.push(path);
    }

    dirs.sort();

    Ok(dirs)
}

fn get_pid(buf: &path::Path) -> Result<u32> {
    let path = buf.join("supervise").join("pid");
    let data: u32 = fs::read_to_string(path)?.trim().parse()?;

    Ok(data)
}

fn get_state(buf: &path::Path) -> Result<ServiceState> {
    let path = buf.join("supervise").join("stat");
    let s = fs::read_to_string(path)?;

    match s.trim() {
        "run" => Ok(ServiceState::Run),
        "down" => Ok(ServiceState::Down),
        "finish" => Ok(ServiceState::Finish),
        _ => Err(anyhow!("unknown service state: '{:?}'", s)),
    }
}
