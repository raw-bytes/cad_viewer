use std::fmt;
use std::fmt::Display;

use nalgebra_glm as glm;

/// An AABB bounding volume
#[derive(Debug, Clone)]
pub struct BBox {
    /// the corner with the lower coordinates
    pub min: glm::Vec3,
    /// the corner with the upper coordinates
    pub max: glm::Vec3,
}

impl BBox {
    /// Creates a new empty bounding volume
    pub fn new() -> Self {
        let min = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
        let max = glm::vec3(f32::MIN, f32::MIN, f32::MIN);

        BBox { min, max }
    }

    /// Returns true if the bbox is empty and false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Extends the bounding volume with the given position
    ///
    ///* `p` - The position about which the volume is extended
    pub fn extend_pos(&mut self, p: &glm::Vec3) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Extends the bounding volume with the given position
    ///
    ///* `rhs` - The right-hand-side bounding volume about which the volume is extended
    pub fn extend_bbox(&mut self, rhs: &BBox) {
        self.min.x = self.min.x.min(rhs.min.x);
        self.min.y = self.min.y.min(rhs.min.y);
        self.min.z = self.min.z.min(rhs.min.z);

        self.max.x = self.max.x.max(rhs.max.x);
        self.max.y = self.max.y.max(rhs.max.y);
        self.max.z = self.max.z.max(rhs.max.z);
    }

    /// Computes and returns the bounding box center
    #[inline]
    pub fn get_center(&self) -> glm::Vec3 {
        let center = (self.min + self.max) / 2.0;
        center
    }

    /// Computes and returns the bounding box size
    #[inline]
    pub fn get_size(&self) -> glm::Vec3 {
        let size = self.max - self.min;
        size
    }

    /// Returns a reference onto the minimum
    #[inline]
    pub fn get_min(&self) -> &glm::Vec3 {
        &self.min
    }

    /// Returns a reference onto the maximum
    #[inline]
    pub fn get_max(&self) -> &glm::Vec3 {
        &self.max
    }
}

impl Default for BBox {
    #[inline]
    fn default() -> Self {
        BBox::new()
    }
}

fn vec3_to_string(f: &mut fmt::Formatter<'_>, v: &glm::Vec3) -> fmt::Result {
    write!(f, "({}, {}, {})", v[0], v[1], v[2])
}

impl Display for BBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        vec3_to_string(f, &self.min)?;
        write!(f, "-")?;
        vec3_to_string(f, &self.max)
    }
}
