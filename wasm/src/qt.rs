use std::ops::Add;

use na::Point2;
use nalgebra as na;

#[derive(Clone, Copy)]
pub struct Rect {
    start: Point2<f64>,
    end: Point2<f64>,
}

impl Rect {
    fn new(start: Point2<f64>, end: Point2<f64>) -> Self {
        Self { start, end }
    }

    fn contains(&self, point: &Point2<f64>) -> bool {
        *point > self.start && *point < self.end
    }
}

#[derive(Clone, Copy)]
pub struct WeightedPoint {
    pos: Point2<f64>,
    mass: u32,
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

impl Add for WeightedPoint {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            pos: self.pos + rhs.pos.coords,
            mass: self.mass + rhs.mass,
        }
    }
}

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
        if depth > 10 {
            return false;
        }

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
                point: ref self_point,
            } => {
                if !boundary.contains(&self_point.pos) {
                    return false;
                }
                let mut children = self.subdivide();
                let inserted = children.iter_mut().any(|c| c.insert(point, depth + 1));
                if inserted {
                    *self = Self::Internal {
                        boundary,
                        children,
                        cm: WeightedPoint::default(),
                    };
                }
                inserted
            }
            Self::Internal {
                boundary, children, ..
            } => {
                if !boundary.contains(point) {
                    return false;
                }
                children.iter_mut().any(|c| c.insert(point, depth + 1))
            }
        }
    }

    /// Calculate the center of mass for each internal node
    pub fn com(&mut self) -> Option<WeightedPoint> {
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
                    .filter_map(|c| c.com())
                    .reduce(|acc, p| {
                        WeightedPoint::new(acc.pos + p.pos.coords, acc.mass + p.mass)
                    })?;

                cm.pos /= 4.;
                *self_cm = cm;
                Some(cm)
            }
        }
    }

    fn boundary(&self) -> &Rect {
        match self {
            Self::Empty { boundary } => boundary,
            Self::External { boundary, .. } => boundary,
            Self::Internal { boundary, .. } => boundary,
        }
    }

    fn subdivide(&self) -> [Box<Self>; 4] {
        let boundary = self.boundary();
        let center = na::center(&boundary.start, &boundary.end);
        let diff = center - boundary.start;
        let diff_x = na::vector![diff.x, 0.];
        let diff_y = na::vector![0., diff.y];

        [
            Box::new(Self::Empty {
                boundary: Rect::new(boundary.start, center),
            }),
            Box::new(Self::Empty {
                boundary: Rect::new(boundary.start + diff_x, center + diff_x),
            }),
            Box::new(Self::Empty {
                boundary: Rect::new(boundary.start + diff_y, center + diff_y),
            }),
            Box::new(Self::Empty {
                boundary: Rect::new(center, boundary.end),
            }),
        ]
    }
}
