use rust_expression::Calculator;

use linefeed::{ Interface, ReadResult };

use std::io;
use std::sync::Arc;

const HISTORY_FILE: &str = "rust-expression-repl.hst";

fn main() -> io::Result<()> {
    let interface = Arc::new(Interface::new("Calc")?);

    println!("This is the rust-expression repl program.");
    println!("Press Ctrl-D or \"quit\" to exit.");
    println!("");

    //interface.set_completer(Arc::new(DemoCompleter));
    interface.set_prompt("% > ")?;

    if let Err(e) = interface.load_history(HISTORY_FILE) {
        if e.kind() == io::ErrorKind::NotFound {
            println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
        } else {
            eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
        }
    }

    let mut calc = Calculator::new();

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }
        
        if "quit" == line {
            break;
        }

        match calc.execute(&line) {
            Ok(Some(num)) => println!("{:?}", num),
            Ok(None) => (),
            Err(err) => println!("Error: {:?}", err),
        }
    }

    println!("Goodbye.");

    Ok(())
}