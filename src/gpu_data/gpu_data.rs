use std::{collections::HashMap, rc::Rc};

use anyhow::Result;
use cad_import::{
    structure::{CADData, Material, Node, Shape},
    ID,
};
use glow::HasContext;
use nalgebra_glm::Mat4;

use super::gpu_mesh::GPUMesh;

pub struct GPUMeshWithMaterial<C: HasContext> {
    pub material: Rc<Material>,
    pub mesh: GPUMesh<C>,
}

pub struct GPUShape<C: HasContext> {
    pub parts: Vec<GPUMeshWithMaterial<C>>,
}

pub struct GPUShapeInstance {
    pub transform: Mat4,
    pub shape_index: usize,
}

/// All data on the GPU
pub struct GPUData<C: HasContext> {
    shapes: Vec<GPUShape<C>>,
    instances: Vec<GPUShapeInstance>,
}

impl<C: HasContext> GPUData<C> {
    /// Creates a new empty GPU data object.
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            instances: Vec::new(),
        }
    }

    /// Adds the given cad data to the gpu data.
    ///
    /// # Arguments
    /// * `context` - The GLOW context used for initializing all GPU data.
    /// * `cad_data` - The CAD data to add.
    pub fn add_cad_data(&mut self, context: &C, cad_data: &CADData) -> Result<()> {
        let root_node = cad_data.get_root_node();
        let traversal_context = TraversalContext::new(root_node);
        let mut traversal_data = TraversalData::new();
        self.traverse(context, root_node, traversal_context, &mut traversal_data)?;

        Ok(())
    }

    /// Returns a reference onto the internally stored shapes.
    pub fn get_shapes(&self) -> &[GPUShape<C>] {
        &self.shapes
    }

    /// Returns a reference onto the shape instances.
    pub fn get_instances(&self) -> &[GPUShapeInstance] {
        &self.instances
    }

    /// Internal function for traversing over the node structure and copying all data to GPU.
    ///
    /// # Arguments
    /// * `context` - The GLOW context used for initializing all GPU data.
    /// * `node` - The currently visited node.
    /// * `traversal_context` - Additional information used during traversal.
    fn traverse(
        &mut self,
        context: &C,
        node: &Node,
        traversal_context: TraversalContext,
        traversal_data: &mut TraversalData,
    ) -> Result<()> {
        let shapes = node.get_shapes();

        for shape in shapes {
            let shape_index = self.get_shape_index(context, shape, traversal_data)?;

            self.instances.push(GPUShapeInstance {
                transform: traversal_context.transform,
                shape_index,
            });
        }

        // traverse over the children of the current node
        for child in node.get_children().iter() {
            // compute new transform
            let child_traversal_context = traversal_context.derive(child);

            self.traverse(context, child, child_traversal_context, traversal_data)?;
        }

        Ok(())
    }

    /// Returns an index for the given shape
    ///
    /// # Arguments
    /// * `context` - The GLOW context.
    /// * `shape` - The CPU
    fn get_shape_index(
        &mut self,
        context: &C,
        shape: &Shape,
        traversal_data: &mut TraversalData,
    ) -> Result<usize> {
        let shape_id = shape.get_id();

        // check if a GPU shape for this shape already
        match traversal_data.shape_map.get(&shape_id) {
            Some(index) => {
                return Ok(*index);
            }
            None => {}
        }

        let index = traversal_data.shape_map.len();

        let gpu_shape = Self::create_gpu_shape(context, shape)?;
        self.shapes.push(gpu_shape);

        Ok(index)
    }

    /// Creates a GPU shape based on the given CPU shape.
    ///
    /// # Arguments
    /// * `context` - The GLOW context.
    /// * `shape` - The CPU shape.
    fn create_gpu_shape(context: &C, shape: &Shape) -> Result<GPUShape<C>> {
        let mut parts = Vec::with_capacity(shape.get_parts().len());

        for part in shape.get_parts() {
            let material = part.get_material();

            let gpu_mesh = GPUMesh::new(context, part.get_mesh().as_ref())?;

            let gpu_part = GPUMeshWithMaterial {
                material: material.clone(),
                mesh: gpu_mesh,
            };

            parts.push(gpu_part);
        }

        Ok(GPUShape { parts })
    }
}

/// Contextual data used during traversing the node data.
#[derive(Clone)]
struct TraversalContext {
    /// The current transformation matrix
    transform: Mat4,
}

impl TraversalContext {
    /// Returns a new empty traversal context.
    pub fn new(root_node: &Node) -> Self {
        let transform: Mat4 = match root_node.get_transform() {
            Some(t) => t,
            None => Mat4::identity(),
        };

        Self { transform }
    }

    /// Returns a new traversal context by visiting the given node.
    ///
    /// # Arguments
    /// * `node` - The node to visit based on the current traversal context
    pub fn derive(&self, node: &Node) -> Self {
        let mut result = self.clone();

        match node.get_transform() {
            Some(t) => {
                result.transform *= t;
            }
            None => {}
        }

        result
    }
}

struct TraversalData {
    pub shape_map: HashMap<ID, usize>,
}

impl TraversalData {
    pub fn new() -> Self {
        Self {
            shape_map: HashMap::new(),
        }
    }
}
