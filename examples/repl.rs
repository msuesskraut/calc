use rust_expression::{Calculator, Plot, Range, Value};

use linefeed::{Interface, ReadResult};

use std::io;
use std::sync::Arc;

fn draw(calculator: &Calculator, plot: &Plot) {
    const WIDTH: i32 = 60;
    const HEIGHT: i32 = 25;

    let mut chart = vec![vec![' '; WIDTH as usize]; HEIGHT as usize];
    let x_screen = Range::new(0, WIDTH);
    let y_screen = Range::new(0, HEIGHT);

    for w in 0..WIDTH {
        let x = x_screen.project(w, &plot.x_range).unwrap();
        if let Some(Some(y)) = calculator.calc(x, plot).map(|y| plot.y_range.project(y, &y_screen)) {
            let h = HEIGHT - (y as i32);
            if h < HEIGHT {
                chart[h as usize][w as usize] = '*';
            }
        }
    }

    for line in chart {
        let mut s = String::with_capacity(WIDTH as usize);
        for ch in line {
            s.push(ch);
        }
        println!("{}", s);
    }
}

fn main() -> io::Result<()> {
    let interface = Arc::new(Interface::new("Calc")?);

    println!("This is the rust-expression repl program.");
    println!("Press Ctrl-D or \"quit\" to exit.");
    println!("");

    interface.set_prompt("% > ")?;

    let mut calc = Calculator::new();

    while let ReadResult::Input(line) = interface.read_line()? {
        if !line.trim().is_empty() {
            interface.add_history_unique(line.clone());
        }

        if "quit" == line {
            break;
        }

        match calc.execute(&line) {
            Ok(Value::Number(num)) => println!("{:}", num),
            Ok(Value::Void) => (),
            Ok(Value::Solved { variable, value }) => println!("{:} = {:}", variable, value),
            Ok(Value::Plot(plot)) => draw(&calc, &plot),
            Err(err) => println!("Error: {:}", err),
        }
    }

    println!("Goodbye.");

    Ok(())
}
