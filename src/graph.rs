use crate::calc::{calc_operand, Env};
use crate::{
    ast::{Function, Number},
    calc::TopLevelEnv,
};

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
struct Graph<'a> {
    fun: &'a Function,
}

impl<'a> Graph<'a> {
    fn x_name(&self) -> &str {
        &self.fun.args[0]
    }

    fn calc(&self, x: Number, env: &dyn Env) -> Option<Number> {
        let call_env = ArgEnv {
            name: self.x_name(),
            value: x,
            env,
        };
        calc_operand(&self.fun.body, &call_env).ok()
    }
}

#[derive(Debug, PartialEq, Clone)]
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

    pub fn get_distance(&self) -> Number {
        self.max - self.min
    }

    pub fn project(&self, pixel: Number, to: &Range) -> Option<Number>
    {
        if (self.min..self.max).contains(&pixel) {
            Some(
                to.min + (((pixel - self.min) / self.get_distance()) * to.get_distance()),
            )
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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
}

#[derive(Debug, PartialEq)]
pub struct Plot {
    pub graph: Vec<Option<Number>>,
}

impl Plot {
    pub fn new(name: &str, env: &TopLevelEnv, area: &Area, screen: &Area) -> Result<Plot, GraphError> {
        let fun = Graph { fun: env
            .get_fun(name)
            .ok_or_else(|| GraphError::UnknownFunction(name.to_string()))?};
        let graph = ((screen.x.min as i32)..(screen.x.max as i32)).map(|w| {
            let x = screen.x.project(w as f64, &area.x).unwrap();
            if let Some(Some(y)) = fun.calc(x, env).map(|y| area.y.project(y, &screen.y)) {
                Some(y)
            } else {
                None
            }
        }).collect();
        Ok(Plot {
            graph
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
        let graph = Graph { fun: &fun };
        assert_eq!(Some(1.0), graph.calc(1.0, &env));
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
        let area = Area::new(-100., -100., 100., 100.);
        let screen = Area::new(0., 0., 40., 40.);
        let plot = Plot::new("f", &env, &area, &screen).unwrap();
        assert_eq!(40, plot.graph.len());
        assert_eq!(None, plot.graph[0]);
        assert_eq!(Some(18.0), plot.graph[19]);
        assert_eq!(None, plot.graph[39]);
    }
}
