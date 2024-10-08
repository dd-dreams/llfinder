mod find;
use std::{
    env::args as args_fn,
    io::Seek
};

const HELP: &str = "llfind {} - find dynamically linked libraries in binaries

Usage:
    llfind <path1> <path2> ...

Positional arguments:
    path1, path2, ...
";

fn main() {
    let args = args_fn();
    let version = env!("CARGO_PKG_VERSION");
    if args.len() == 1 {
        print!("{}", HELP.replace("{}", version));
        return;
    }

    for path in args.skip(1) {
        let mut file = std::fs::File::open(&path).expect(&format!("Invalid path {}", path));
        println!("-- {path} --");

        file.rewind().expect("Could not rewind");

        match find::fileos(&mut file).unwrap() {
            (find::FileType::Macho, bits) => {
                file.rewind().expect("Could not rewind");
                let libs = find::find_macho(&mut file, bits, 0).expect("IO Error");
                for lib in libs {
                    println!("{}, compatibility version: {}, current version: {}, load: {}",
                        lib.path,
                        lib.compat_ver,
                        lib.curr_ver,
                        if lib.cmd == 1 {"full path"} else if lib.cmd == 0 {"current directory load"} else {"not required"});
                }
            }
            (find::FileType::MachoM, _) => {
                let archs = find::find_multi_macho(&mut file).expect("IO Error");
                for arch in archs {
                    println!("-- {path} {:?} --", arch.cpu_type);
                    for lib in arch.libs {
                        println!("{}, compatibility version: {}, current version: {}, load: {}",
                            lib.path,
                            lib.compat_ver,
                            lib.curr_ver,
                            if lib.cmd == 1 {"full path"} else if lib.cmd == 0 {"current directory load"} else {"not required"});
                    }
                }
            }
            (find::FileType::ELF, _) => {
                let libs = find::find_elf(&mut file).unwrap();
                for lib in libs {
                    println!("{}", lib.name);
                }
            }
            (find::FileType::PE, _) => {
                let libs = find::find_pe(&mut file).unwrap();
                for lib in libs {
                    println!("{}", lib.name);
                }
            }
            _ => {
                println!("Unknown platform for {}", path);
            }
        }
    }
}
