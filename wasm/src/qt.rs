#![allow(unused)]

use std::ops::Add;

use na::{Point2, Vector2};
use nalgebra as na;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::sim::Particle;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    start: Point2<f64>,
    center: Point2<f64>,
    end: Point2<f64>,
}

impl Rect {
    fn new(start: Point2<f64>, end: Point2<f64>) -> Self {
        Self {
            start,
            center: na::center(&start, &end),
            end,
        }
    }

    /// Check if a point exists within the rect
    fn contains(&self, point: &Point2<f64>) -> bool {
        *point >= self.start && *point <= self.end
    }

    /// Calculate the "width", 1/4 of the perimeter
    fn width(&self) -> f64 {
        let diff = self.end - self.start;
        let perimeter = diff.x * 2. + diff.y * 2.;
        perimeter / 4.
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WeightedPoint {
    pub pos: Point2<f64>,
    pub mass: u32,
}

impl WeightedPoint {
    fn new(pos: Point2<f64>, mass: u32) -> Self {
        Self { pos, mass }
    }
}

impl Default for WeightedPoint {
    fn default() -> Self {
        Self {
            pos: Point2::origin(),
            mass: 0,
        }
    }
}

#[derive(Debug)]
pub enum QuadTree {
    Internal {
        boundary: Rect,
        children: [Box<Self>; 4],
        cm: WeightedPoint,
    },
    External {
        boundary: Rect,
        point: WeightedPoint,
    },
    Empty {
        boundary: Rect,
    },
}

impl QuadTree {
    pub fn new(world: Point2<f64>) -> Self {
        Self::Empty {
            boundary: Rect::new(Point2::origin(), world),
        }
    }

    /// Insert a point into the quadtree
    pub fn insert(&mut self, point: &Point2<f64>, depth: u8) -> bool {
        match self {
            &mut Self::Empty { boundary } => {
                if !boundary.contains(point) {
                    return false;
                }
                *self = Self::External {
                    boundary,
                    point: WeightedPoint::new(*point, 1),
                };
                true
            }
            &mut Self::External {
                boundary,
                point: mut p,
            } => {
                if !boundary.contains(point) {
                    return false;
                }
                // After a certain depth assume identical position
                if depth > 6 {
                    p.mass += 1;
                    return true;
                }
                let mut children = self.subdivide();
                let inserted = !children
                    .iter_mut()
                    .map(|c| c.insert(&p.pos, depth + 1))
                    .all(|c| !c);
                if inserted {
                    *self = Self::Internal {
                        boundary,
                        children,
                        cm: WeightedPoint::default(),
                    };
                }
                inserted && self.insert(point, depth)
            }
            Self::Internal {
                boundary, children, ..
            } => {
                if !boundary.contains(point) {
                    return false;
                }
                !children
                    .iter_mut()
                    .map(|c| c.insert(point, depth + 1))
                    .all(|c| !c)
            }
        }
    }

    /// Calculate the center of mass for each internal node
    pub fn measure_nodes(&mut self) -> Option<WeightedPoint> {
        match self {
            Self::Empty { .. } => None,
            Self::External { point, .. } => Some(*point),
            Self::Internal {
                children,
                cm: self_cm,
                ..
            } => {
                let mut cm = children
                    .iter_mut()
                    .filter_map(|c| c.measure_nodes())
                    .reduce(|acc, p| {
                        WeightedPoint::new(
                            acc.pos + (p.pos.coords * p.mass as f64),
                            acc.mass + p.mass,
                        )
                    })?;

                cm.pos /= cm.mass as f64;
                *self_cm = cm;
                Some(cm)
            }
        }
    }

    /// Perform Barnes-Hut approximation for a particle, returns a list of weighted points whose granularity is determined by the parameter theta.
    pub fn approximate_points(
        &self,
        particle: &Particle,
        theta: f64,
    ) -> Option<Vec<WeightedPoint>> {
        match self {
            Self::Empty { .. } => None,
            Self::External { point: p, .. } => {
                if na::distance(&particle.pos, &p.pos) > particle.aoe {
                    return None;
                }
                Some(vec![*p])
            }
            Self::Internal {
                boundary,
                children,
                cm,
                ..
            } => {
                let w = boundary.width();
                let d = na::distance(&particle.pos, &boundary.center);
                if w / d < theta {
                    if na::distance(&particle.pos, &cm.pos) > particle.aoe {
                        return None;
                    }
                    Some(vec![*cm])
                } else {
                    Some(
                        children
                            .iter()
                            .map(|c| c.approximate_points(particle, theta))
                            .flatten()
                            .flatten()
                            .collect(),
                    )
                }
            }
        }
    }

    /// Return the point at the center of the boundary
    pub fn center(&self) -> &Point2<f64> {
        &self.boundary().center
    }

    pub fn render(&self, cx: &CanvasRenderingContext2d, depth: u8) {
        match self {
            Self::Empty { boundary } => {
                let dim = boundary.end - boundary.start;
                cx.stroke_rect(boundary.start.x, boundary.start.y, dim.x, dim.y);
            }
            Self::External { boundary, .. } => {
                let dim = boundary.end - boundary.start;
                cx.stroke_rect(boundary.start.x, boundary.start.y, dim.x, dim.y);
            }
            Self::Internal {
                boundary, children, ..
            } => {
                let dim = boundary.end - boundary.start;
                cx.stroke_rect(boundary.start.x, boundary.start.y, dim.x, dim.y);
                children.iter().for_each(|c| {
                    c.render(cx, depth + 1);
                });
            }
        }
    }

    /// Get the boundary rect out of the enum
    fn boundary(&self) -> &Rect {
        match self {
            Self::Empty { boundary } => boundary,
            Self::External { boundary, .. } => boundary,
            Self::Internal { boundary, .. } => boundary,
        }
    }

    /// Chop the node into four quarters and return the new subnodes
    fn subdivide(&self) -> [Box<Self>; 4] {
        let &Rect { start, center, end } = self.boundary();
        let diff = center - start;
        let diff_x = na::vector![diff.x, 0.];
        let diff_y = na::vector![0., diff.y];

        [
            Box::new(Self::Empty {
                boundary: Rect::new(start, center),
            }),
            Box::new(Self::Empty {
                boundary: Rect::new(start + diff_x, center + diff_x),
            }),
            Box::new(Self::Empty {
                boundary: Rect::new(start + diff_y, center + diff_y),
            }),
            Box::new(Self::Empty {
                boundary: Rect::new(center, end),
            }),
        ]
    }
}
