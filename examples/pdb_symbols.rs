extern crate getopts;
extern crate mozpdb;

use mozpdb as pdb;
use getopts::Options;
use pdb::FallibleIterator;
use std::env;
use std::io::Write;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} input.pdb", program);
    print!("{}", opts.usage(&brief));
}

fn print_row<'p>(segment: u16, offset: u32, kind: &'static str, name: pdb::RawString<'p>) {
    println!("{:x}\t{:x}\t{}\t{}", segment, offset, kind, name.to_string());
}

fn print_symbol(symbol: &pdb::Symbol) -> pdb::Result<()> {
    match symbol.parse()? {
        pdb::SymbolData::PublicSymbol(data) => {
            print_row(data.segment, data.offset, "function", symbol.name()?);
        }
        pdb::SymbolData::DataSymbol(data) => {
            print_row(data.segment, data.offset, "data", symbol.name()?);
        }
        pdb::SymbolData::Procedure(data) => {
            print_row(data.segment, data.offset, "function", symbol.name()?);
        }
        _ => {
            // ignore everything else
        }
    }

    Ok(())
}

fn walk_symbols(mut symbols: pdb::SymbolIter) -> pdb::Result<()> {
    println!("segment\toffset\tkind\tname");

    while let Some(symbol) = symbols.next()? {
        match print_symbol(&symbol) {
            Ok(_) => {}
            Err(e) => {
                writeln!(&mut std::io::stderr(), "error printing symbol {:?}: {}", symbol, e)
                    .expect("stderr write");
            }
        }
    }

    Ok(())
}

fn dump_pdb(filename: &str) -> pdb::Result<()> {
    let file = std::fs::File::open(filename)?;
    let mut pdb = pdb::PDB::open(file)?;
    let symbol_table = pdb.global_symbols()?;
    println!("Global symbols:");
    walk_symbols(symbol_table.iter())?;

    println!("Module private symbols:");
    let dbi = pdb.debug_information()?;
    let mut modules = dbi.modules()?;
    while let Some(module) = modules.next()? {
        println!("Module: {}", module.object_file_name());
        let info = pdb.module_info(&module)?;
        walk_symbols(info.symbols()?)?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let filename = if matches.free.len() == 1 {
        &matches.free[0]
    } else {
        print_usage(&program, opts);
        return;
    };

    match dump_pdb(&filename) {
        Ok(_) => {}
        Err(e) => {
            writeln!(&mut std::io::stderr(), "error dumping PDB: {}", e)
                .expect("stderr write");
        }
    }
}
