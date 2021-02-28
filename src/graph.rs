use crate::calc::{calc_operand, Env};
use crate::{
    ast::{Function, Number},
    calc::TopLevelEnv,
};

use num::iter::range_step_from;

use thiserror::Error;

use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GraphError {
    #[error("Unknown function `{0}` to plot")]
    UnknownFunction(String),
}

struct ArgEnv<'a> {
    name: &'a str,
    value: Number,
    env: &'a dyn Env,
}

impl<'a> Env for ArgEnv<'a> {
    fn get(&self, sym: &str) -> Option<&Number> {
        if sym == self.name {
            Some(&self.value)
        } else {
            self.env.get(sym)
        }
    }

    fn get_fun(&self, fun: &str) -> Option<&Function> {
        self.env.get_fun(fun)
    }
}

#[derive(Debug, PartialEq)]
pub struct Graph {
    env: TopLevelEnv,
    fun: Function,
}

impl Graph {
    pub fn new(name: &str, env: &TopLevelEnv) -> Result<Graph, GraphError> {
        let env = env.clone();
        let graph = Graph {
            fun: env
                .get_fun(name)
                .ok_or_else(|| GraphError::UnknownFunction(name.to_string()))?
                .clone(),
            env,
        };

        Ok(graph)
    }

    fn x_name(&self) -> &str {
        &self.fun.args[0]
    }

    fn calc(&self, x: Number) -> Option<Number> {
        let call_env = ArgEnv {
            name: self.x_name(),
            value: x,
            env: &self.env,
        };
        calc_operand(&self.fun.body, &call_env).ok()
    }

    pub fn plot(&self, area: &Area, screen: &Area) -> Result<Plot, GraphError> {
        Plot::new(self, area, screen)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Range {
    pub min: Number,
    pub max: Number,
}

impl Range {
    pub fn new(min: Number, max: Number) -> Range {
        if min >= max {
            panic!(format!("min {:?} must be smaller than max {:?}", min, max));
        }
        Range { min, max }
    }

    pub fn is_in_range(&self, pos: Number) -> bool {
        return self.min <= pos && pos <= self.max;
    }

    pub fn get_distance(&self) -> Number {
        self.max - self.min
    }

    pub fn project(&self, pixel: Number, to: &Range) -> Option<Number> {
        if (self.min..self.max).contains(&pixel) {
            Some(to.min + (((pixel - self.min) / self.get_distance()) * to.get_distance()))
        } else {
            None
        }
    }

    pub fn move_by(&mut self, delta: Number) {
        self.min += delta;
        self.max += delta;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Area {
    pub x: Range,
    pub y: Range,
}

impl Area {
    pub fn new(x_min: Number, y_min: Number, x_max: Number, y_max: Number) -> Area {
        Area {
            x: Range::new(x_min, x_max),
            y: Range::new(y_min, y_max),
        }
    }

    pub fn move_by(&mut self, x_delta: Number, y_delta: Number) {
        self.x.move_by(x_delta);
        self.y.move_by(y_delta);
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Tic {
    pos: Number,
    label: Number,
}

impl Tic {
    pub fn new(pos: Number, label: Number) -> Tic {
        Tic { pos, label }
    }

    pub fn create_tics(screen: &Range, area: &Range) -> Vec<Tic> {
        let width = area.max - area.min;
        let tic_width = 10f64.powf((width.log10() - 1.0).round());
        if area.is_in_range(0.0) {
            let left: Vec<Tic> = range_step_from(0f64, -tic_width)
                .take_while(|label| label > &area.min)
                .map(|label| Tic::new(area.project(label, screen).unwrap(), label))
                .collect();
            let right: Vec<Tic> = range_step_from(0f64, tic_width)
                .take_while(|label| label < &area.max)
                .map(|label| Tic::new(area.project(label, screen).unwrap(), label))
                .collect();
            left.iter()
                .rev()
                .chain(right.iter())
                .map(|tic| *tic)
                .collect()
        } else {
            let start = (area.min / tic_width).ceil() * tic_width;
            range_step_from(start, tic_width)
                .take_while(|label| label < &area.max)
                .map(|label| Tic::new(area.project(label, screen).unwrap(), label))
                .collect()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Axis {
    pos: Number,
    tics: Vec<Tic>,
}

impl Axis {
    pub fn new(pos: Option<Number>, screen: &Range, area: &Range) -> Option<Axis> {
        pos.map(|pos| Axis {
            pos,
            tics: Tic::create_tics(screen, area),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Plot {
    pub points: Vec<Option<Number>>,
    pub screen: Area,
    pub x_axis: Option<Axis>,
    pub y_axis: Option<Axis>,
}

impl Plot {
    pub fn new(graph: &Graph, area: &Area, screen: &Area) -> Result<Plot, GraphError> {
        let points = ((screen.x.min as i32)..(screen.x.max as i32))
            .map(|w| {
                let x = screen.x.project(w as f64, &area.x).unwrap();
                if let Some(Some(y)) = graph.calc(x).map(|y| area.y.project(y, &screen.y)) {
                    Some(y)
                } else {
                    None
                }
            })
            .collect();
        let x_axis = Axis::new(area.y.project(0., &screen.y), &screen.y, &area.y);
        let y_axis = Axis::new(area.x.project(0., &screen.x), &screen.x, &area.x);

        Ok(Plot {
            points,
            screen: *screen,
            x_axis,
            y_axis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Operand, Operation, Term};
    use crate::calc::TopLevelEnv;

    #[test]
    fn function_arg_x() {
        let mut env = TopLevelEnv::default();
        env.put("x".to_string(), -19.0);
        let name = "x";
        let value = 42.0;
        let env = ArgEnv {
            name,
            value,
            env: &env,
        };
        assert_eq!(Some(&42.0), env.get("x"));
    }

    #[test]
    fn function_arg_y() {
        let mut env = TopLevelEnv::default();
        env.put("y".to_string(), -19.0);
        let name = "x";
        let value = 42.0;
        let env = ArgEnv {
            name,
            value,
            env: &env,
        };
        assert_eq!(Some(&-19.0), env.get("y"));
    }

    #[test]
    fn function_call() {
        let fun = Function {
            args: vec!["x".to_string()],
            body: Operand::Symbol("x".to_string()),
        };
        let env = TopLevelEnv::default();
        let graph = Graph { fun: fun, env };
        assert_eq!(Some(1.0), graph.calc(1.0));
    }

    #[test]
    #[should_panic(expected = "min 4.0 must be smaller than max 3.0")]
    fn range_construct_failure() {
        let _ = Range::new(4., 3.);
    }

    #[test]
    fn range_distance_f64() {
        assert_eq!(4.0, Range::new(10.0, 14.0).get_distance());
    }

    #[test]
    fn range_project_plot_to_screen() {
        let plot = Range::new(-100., 100.);
        let screen = Range::new(0., 400.);

        assert_eq!(Some(200.0), plot.project(0., &screen));
        assert_eq!(Some(300.0), plot.project(50., &screen));
        assert_eq!(Some(100.0), plot.project(-50., &screen));
    }

    #[test]
    fn range_project_plot_to_screen_out_of_range() {
        let plot = Range::new(-100., 100.);
        let screen = Range::new(0., 400.);

        assert_eq!(None, plot.project(-101., &screen));
        assert_eq!(None, plot.project(100., &screen));
    }

    #[test]
    fn range_project_screen_to_plot() {
        let screen = Range::new(0., 400.);
        let plot = Range::new(-100., 100.);

        assert_eq!(Some(-100.0), screen.project(0., &plot));
        assert_eq!(Some(-50.0), screen.project(100., &plot));
        assert_eq!(Some(0.0), screen.project(200., &plot));
        assert_eq!(Some(50.0), screen.project(300., &plot));
        assert_eq!(Some(99.5), screen.project(399., &plot));
    }

    #[test]
    fn range_project_screen_to_plot_out_of_range() {
        let screen = Range::new(0., 400.);
        let plot = Range::new(-100., 100.);

        assert_eq!(None, screen.project(-1., &plot));
        assert_eq!(None, screen.project(400., &plot));
    }

    #[test]
    fn construct_plot() {
        let mut env = TopLevelEnv::default();
        let term = {
            let lhs = Operand::Symbol("x".to_string());
            let rhs = Operand::Number(2.0);
            let op = Operation::Mul;
            Term { lhs, rhs, op }
        };
        let body = Operand::Term(Box::new(term));
        let fun = Function {
            args: vec!["x".to_string()],
            body,
        };
        env.put_fun("f".to_string(), fun);
        let graph = Graph::new("f", &env).unwrap();
        let area = Area::new(-100., -100., 100., 100.);
        let screen = Area::new(0., 0., 40., 40.);
        let plot = graph.plot(&area, &screen).unwrap();

        assert_eq!(20., plot.x_axis.unwrap().pos);
        assert_eq!(20., plot.y_axis.unwrap().pos);
        assert_eq!(40, plot.points.len());
        assert_eq!(None, plot.points[0]);
        assert_eq!(Some(18.0), plot.points[19]);
        assert_eq!(None, plot.points[39]);
    }
}
