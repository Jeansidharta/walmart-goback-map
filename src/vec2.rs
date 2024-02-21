use serde::Serialize;

#[derive(Debug, Serialize, Default, Clone)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn scale(&self, factor: f64) -> Vec2 {
        Vec2 {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
    pub fn modulus_sqr(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }

    pub fn add(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn subtract(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn dot(&self, other: &Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance_sqr(&self, other: &Self) -> f64 {
        (other.x - self.x).powi(2) + (other.y - self.y).powi(2)
    }

    pub fn project_line_segment(&self, (p1, p2): (&Vec2, &Vec2)) -> (Vec2, f64) {
        let x = self.subtract(p1);
        let p = p2.subtract(p1);
        let modulus_p_sqr = p.modulus_sqr();
        let dot = x.dot(&p).clamp(0.0, modulus_p_sqr);
        let t = dot / modulus_p_sqr;
        (p.scale(t).add(p1), t)
    }
}
