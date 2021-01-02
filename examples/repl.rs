use rust_expression::{Calculator, Number, Plot, Value};

use linefeed::{Interface, ReadResult};

use std::io;
use std::sync::Arc;

fn draw(calculator: &Calculator, plot: &Plot) {
    const WIDTH: usize = 60;
    const HEIGHT: usize = 25;

    let mut chart = vec![vec![' '; WIDTH]; HEIGHT];

    for w in 0..WIDTH {
        let x = plot.x_range.min
            + ((plot.x_range.max - plot.x_range.min) / (WIDTH as Number) * (w as Number));
        if let Some(y) = calculator.calc(x, plot) {
            if y >= plot.y_range.min && y <= plot.y_range.max {
                let h = HEIGHT
                    - ((y - plot.y_range.min)
                        / ((plot.y_range.max - plot.y_range.min) / (HEIGHT as Number)))
                        as usize;
                if w < WIDTH && h < HEIGHT {
                    chart[h][w] = '*';
                }
            }
        }
    }

    for line in chart {
        let mut s = String::with_capacity(WIDTH);
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
