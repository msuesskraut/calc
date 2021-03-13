use crate::{
    ast::{Function, Number},
    calc::{calc_operand, Env, TopLevelEnv},
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
        match self.fun {
            Function::Custom(ref fun) => &fun.args[0],
            Function::BuildIn(ref fun) => &fun.arg,
        }
    }

    fn calc(&self, x: Number) -> Option<Number> {
        match self.fun {
            Function::Custom(ref fun) => {
                let call_env = ArgEnv {
                    name: self.x_name(),
                    value: x,
                    env: &self.env,
                };
                calc_operand(&fun.body, &call_env).ok()
            }
            Function::BuildIn(ref fun) => Some((fun.body)(x)),
        }
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

    pub fn contains(&self, pos: Number) -> bool {
        (self.min..self.max).contains(&pos)
    }

    pub fn get_distance(&self) -> Number {
        self.max - self.min
    }

    pub fn project_inclusive(&self, pixel: Number, to: &Range) -> Option<Number> {
        if self.contains(pixel) {
            Some(self.project(pixel, to))
        } else {
            None
        }
    }

    pub fn project(&self, pixel: Number, to: &Range) -> Number {
        to.min + (((pixel - self.min) / self.get_distance()) * to.get_distance())
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
    pub pos: Number,
    pub label: Number,
}

impl Tic {
    pub fn new(pos: Number, label: Number) -> Tic {
        Tic { pos, label }
    }

    pub fn create_tics(screen: &Range, area: &Range) -> Vec<Tic> {
        let width = area.max - area.min;
        let step = 10f64.powf((width.log10() - 1.0).round());
        if area.contains(0.0) {
            let left: Vec<Tic> = range_step_from(0f64, -step)
                .take_while(|label| label > &area.min)
                .map(|label| Tic::new(area.project_inclusive(label, screen).unwrap(), label))
                .collect();

            let right: Vec<Tic> = range_step_from(step, step)
                .take_while(|label| label < &area.max)
                .map(|label| Tic::new(area.project_inclusive(label, screen).unwrap(), label))
                .collect();

            left.iter().rev().chain(right.iter()).copied().collect()
        } else {
            let start = (area.min / step).ceil() * step;

            range_step_from(start, step)
                .take_while(|label| label < &area.max)
                .map(|label| Tic::new(area.project_inclusive(label, screen).unwrap(), label))
                .collect()
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Axis {
    pub pos: Number,
    pub tics: Vec<Tic>,
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
                let x = screen.x.project_inclusive(w as f64, &area.x).unwrap();
                graph.calc(x).map(|y| area.y.project(y, &screen.y))
            })
            .collect();
        let x_axis = Axis::new(area.y.project_inclusive(0., &screen.y), &screen.x, &area.x);
        let y_axis = Axis::new(area.x.project_inclusive(0., &screen.x), &screen.y, &area.y);

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
    use crate::ast::{CustomFunction, Operand, Operation, Term};
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
        let fun = Function::Custom(CustomFunction {
            args: vec!["x".to_string()],
            body: Operand::Symbol("x".to_string()),
        });
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

        assert_eq!(Some(200.0), plot.project_inclusive(0., &screen));
        assert_eq!(Some(300.0), plot.project_inclusive(50., &screen));
        assert_eq!(Some(100.0), plot.project_inclusive(-50., &screen));
    }

    #[test]
    fn range_project_plot_to_screen_out_of_range() {
        let plot = Range::new(-100., 100.);
        let screen = Range::new(0., 400.);

        assert_eq!(None, plot.project_inclusive(-101., &screen));
        assert_eq!(None, plot.project_inclusive(100., &screen));
    }

    #[test]
    fn range_project_screen_to_plot() {
        let screen = Range::new(0., 400.);
        let plot = Range::new(-100., 100.);

        assert_eq!(Some(-100.0), screen.project_inclusive(0., &plot));
        assert_eq!(Some(-50.0), screen.project_inclusive(100., &plot));
        assert_eq!(Some(0.0), screen.project_inclusive(200., &plot));
        assert_eq!(Some(50.0), screen.project_inclusive(300., &plot));
        assert_eq!(Some(99.5), screen.project_inclusive(399., &plot));
    }

    #[test]
    fn range_project_screen_to_plot_out_of_range() {
        let screen = Range::new(0., 400.);
        let plot = Range::new(-100., 100.);

        assert_eq!(None, screen.project_inclusive(-1., &plot));
        assert_eq!(None, screen.project_inclusive(400., &plot));
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
        let fun = Function::Custom(CustomFunction {
            args: vec!["x".to_string()],
            body,
        });
        env.put_fun("f".to_string(), fun);
        let graph = Graph::new("f", &env).unwrap();
        let area = Area::new(-100., -100., 100., 100.);
        let screen = Area::new(0., 0., 40., 40.);
        let plot = graph.plot(&area, &screen).unwrap();

        assert_eq!(20., plot.x_axis.unwrap().pos);
        assert_eq!(20., plot.y_axis.unwrap().pos);
        assert_eq!(40, plot.points.len());
        assert_eq!(Some(-20.), plot.points[0]);
        assert_eq!(Some(18.), plot.points[19]);
        assert_eq!(Some(58.), plot.points[39]);
    }

    #[test]
    fn create_tics_with_zero() {
        use float_cmp::approx_eq;

        let act = Tic::create_tics(&Range::new(-100., 100.), &Range::new(-5., 15.));
        let exp: Vec<Tic> = range_step_from(-90., 10.)
            .zip(range_step_from(-4., 1.))
            .take(19)
            .map(|(pos, label)| Tic::new(pos, label))
            .collect();

        assert_eq!(exp.len(), act.len());
        assert!(exp.iter().zip(act.iter()).all(|(exp, act)| approx_eq!(
            f64,
            exp.pos,
            act.pos,
            epsilon = 0.00001
        ) && approx_eq!(
            f64,
            exp.label,
            act.label,
            epsilon = 0.00001
        )));
    }

    #[test]
    fn create_tics_above_zero() {
        use float_cmp::approx_eq;

        let act = Tic::create_tics(&Range::new(0., 400.), &Range::new(3., 19.));
        let exp: Vec<Tic> = range_step_from(0., 25.)
            .zip(range_step_from(3., 1.))
            .take(16)
            .map(|(pos, label)| Tic::new(pos, label))
            .collect();

        assert_eq!(exp.len(), act.len());
        println!("{:?}", act);
        assert!(exp.iter().zip(act.iter()).all(|(exp, act)| approx_eq!(
            f64,
            exp.pos,
            act.pos,
            epsilon = 0.00001
        ) && approx_eq!(
            f64,
            exp.label,
            act.label,
            epsilon = 0.00001
        )));
    }

    #[test]
    fn create_tics_below_zero() {
        use float_cmp::approx_eq;

        let act = Tic::create_tics(&Range::new(0., 400.), &Range::new(-19., -3.));
        let exp: Vec<Tic> = range_step_from(0., 25.)
            .zip(range_step_from(-19., 1.))
            .take(16)
            .map(|(pos, label)| Tic::new(pos, label))
            .collect();

        assert_eq!(exp.len(), act.len());
        println!("{:?}", act);
        assert!(exp.iter().zip(act.iter()).all(|(exp, act)| approx_eq!(
            f64,
            exp.pos,
            act.pos,
            epsilon = 0.00001
        ) && approx_eq!(
            f64,
            exp.label,
            act.label,
            epsilon = 0.00001
        )));
    }
}
