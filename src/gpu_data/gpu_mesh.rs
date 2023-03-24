use anyhow::Result;
use cad_import::structure::{IndexData, Mesh, PrimitiveType, Vertices};
use glow::HasContext;

use crate::{gl_call, viewer::gl_call::handle_glow_error};

use super::buffer::Buffer;

type VertexBuffer<C> = Buffer<C, { glow::ARRAY_BUFFER }>;
type IndexBuffer<C> = Buffer<C, { glow::ELEMENT_ARRAY_BUFFER }>;

struct VertexAttributes<C: HasContext> {
    pub position: VertexBuffer<C>,
    pub normal: Option<VertexBuffer<C>>,
}

/// A single GPU mesh defined by vertices and primitives.
pub struct GPUMesh<C: HasContext> {
    vertex_array: C::VertexArray,
    vertices: VertexAttributes<C>,

    primitive_type: u32,
    num_indices: u32,
    indices: Option<IndexBuffer<C>>,
}

impl<C: HasContext> GPUMesh<C> {
    /// Returns a new GPU mesh for the given CPU mesh data.
    ///
    /// # Arguments
    /// * `context` - The GLOW context used for accessing the GPU.
    /// * `mesh` - The CPU mesh data to copy to GPU.
    pub fn new(context: &C, mesh: &Mesh) -> Result<Self> {
        let primitives = mesh.get_primitives();
        let primitive_type = Self::translate_primitive_type(primitives.get_primitive_type())?;
        let num_indices = primitives.get_raw_index_data().num_indices() as u32;

        let vertices = mesh.get_vertices();
        let (vertices, vertex_array) = Self::create_vertex_data(context, vertices)?;

        let indices = match primitives.get_raw_index_data() {
            IndexData::Indices(raw_indices) => {
                let indices = IndexBuffer::<C>::new(context)?;
                indices.set_data(context, raw_indices, super::buffer::Usage::Static);

                Some(indices)
            }
            IndexData::NonIndexed(_) => None,
        };

        Ok(Self {
            vertex_array,
            vertices,
            primitive_type,
            num_indices,
            indices,
        })
    }

    /// Renders the whole GPU mesh.
    pub fn draw(&self, context: &C) {
        gl_call!(context, bind_vertex_array, Some(self.vertex_array));

        match &self.indices {
            Some(indices) => {
                indices.bind(context);
                gl_call!(
                    context,
                    draw_elements,
                    self.primitive_type,
                    self.num_indices as i32,
                    glow::UNSIGNED_INT,
                    0
                );
            }
            None => {
                gl_call!(
                    context,
                    draw_arrays,
                    self.primitive_type,
                    0,
                    self.num_indices as i32
                );
            }
        }

        gl_call!(context, bind_vertex_array, None);
    }

    /// Returns true if normals are defined
    pub fn has_normals(&self) -> bool {
        self.vertices.normal.is_some()
    }

    /// Translates the given cad_import primitive type to glow primitive type.
    ///
    /// # Arguments
    /// * `primitive_type` - The cad_import primitive type to translate.
    fn translate_primitive_type(primitive_type: PrimitiveType) -> Result<u32> {
        match primitive_type {
            PrimitiveType::Point => Ok(glow::POINTS),
            PrimitiveType::Line => Ok(glow::LINES),
            PrimitiveType::LineLoop => Ok(glow::LINE_LOOP),
            PrimitiveType::LineStrip => Ok(glow::LINE_STRIP),
            PrimitiveType::Triangles => Ok(glow::TRIANGLES),
            PrimitiveType::TriangleFan => Ok(glow::TRIANGLE_FAN),
            PrimitiveType::TriangleStrip => Ok(glow::TRIANGLE_STRIP),
        }
    }

    /// Creates the vertex array from the given vertex data.
    ///
    /// # Arguments
    /// * `context` - The GLOW context to use for creating the vertex array.
    /// * `vertices` - The vertex data on CPU memory to transfer to the GPU.
    fn create_vertex_data(
        context: &C,
        vertices: &Vertices,
    ) -> Result<(VertexAttributes<C>, C::VertexArray)> {
        // copy position data onto the GPU
        let position = VertexBuffer::<C>::new(context)?;
        position.set_data(
            context,
            vertices.get_positions().as_slice(),
            super::buffer::Usage::Static,
        );

        // normal data
        let normal = match vertices.get_normals() {
            Some(normal_data) => {
                let normal = VertexBuffer::<C>::new(context)?;
                normal.set_data(
                    context,
                    normal_data.as_slice(),
                    super::buffer::Usage::Static,
                );

                Some(normal)
            }
            None => None,
        };

        let vertex_attributes = VertexAttributes { position, normal };

        // initialize vertex array data...
        let vertex_array = handle_glow_error(gl_call!(context, create_vertex_array))?;
        Self::initialize_vertex_array(context, vertex_array, &vertex_attributes);

        gl_call!(context, bind_vertex_array, None);
        Ok((vertex_attributes, vertex_array))
    }

    /// Initializes the vertex array with the given vertex attribute data.
    ///
    /// # Arguments
    /// * `context` - The GLOW context.
    /// * `vertex_array` - The vertex array to initialize.
    /// * `attributes` - The vertex attributes to use to initialize the vertex array.
    fn initialize_vertex_array(
        context: &C,
        vertex_array: C::VertexArray,
        attributes: &VertexAttributes<C>,
    ) {
        gl_call!(context, bind_vertex_array, Some(vertex_array));

        // positions
        attributes.position.bind(context);
        gl_call!(context, enable_vertex_attrib_array, 0);
        gl_call!(
            context,
            vertex_attrib_pointer_f32,
            0,
            3,
            glow::FLOAT,
            false,
            0,
            0
        );

        // normals
        match &attributes.normal {
            Some(normals) => {
                gl_call!(context, enable_vertex_attrib_array, 1);
                normals.bind(context);
                gl_call!(
                    context,
                    vertex_attrib_pointer_f32,
                    0,
                    3,
                    glow::FLOAT,
                    false,
                    0,
                    0
                );
            }
            None => {
                gl_call!(context, disable_vertex_attrib_array, 1);
            }
        }

        gl_call!(context, bind_vertex_array, None);
    }
}
