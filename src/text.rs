use crate::texture;
use wgpu::util::DeviceExt;
use winit::dpi::Size;
use std::collections::HashMap;


#[derive(Default, Debug)]
pub struct TextVecs {
    pub vertices: Vec<CharacterVertex>,
    pub indices: Vec<u32>,
}

pub struct TextBuffers {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub length: u32,
}

impl TextVecs {
    pub fn from_quad(quad: CharacterQuad, offset: Option<u32>) -> Self {
        let offset: u32 = offset.unwrap_or(0);
        let scale: f32 = 1.0 / 8.0;

        let c_y = (quad.character / 8) as f32;
        let c_x = (quad.character % 8) as f32;

        let tl = [c_x * scale, c_y * scale];
        let bl = [c_x * scale, c_y * scale + scale];
        let br = [c_x * scale + scale, c_y * scale + scale];
        let tr = [c_x * scale + scale, c_y * scale];

        let x_1 = quad.position[0];
        let x_2 = quad.position[0] + quad.size[0];
        let y_1 = quad.position[1];
        let y_2 = quad.position[1] - quad.size[1];

        let vertices = vec![
            CharacterVertex { position: [x_1, y_1, 0.0], tex_coords: tl }, // TOP LEFT
            CharacterVertex { position: [x_1, y_2, 0.0], tex_coords: bl }, // BOTTOM LEFT
            CharacterVertex { position: [x_2, y_2, 0.0], tex_coords: br }, // BOTTOM RIGHT
            CharacterVertex { position: [x_2, y_1, 0.0], tex_coords: tr }, // TOP RIGHT
        ];

        let indices = vec![
            1, 3, 0, 1, 2, 3,
            ].iter()
            .map(|x| x + (offset * 4))
            .collect();

        let result = TextVecs {
            vertices,
            indices
        };

        println!("\n{:?}\n", result);

        return result;
    }

    pub fn from_quads(quads: Vec<CharacterQuad>) -> Self {
        quads.iter()
            .fold((Self::default(), 0), |a, e| {
                let mut tv = a.0;
                let offset = a.1;
                
                let mut vecs = Self::from_quad(*e, Some(offset));

                tv.vertices.append(&mut vecs.vertices);
                tv.indices.append(&mut vecs.indices);
                
                (tv, a.1 + 1)
            }).0
    }

    pub fn to_buffers(&self, device: &wgpu::Device) -> TextBuffers {
        let length = self.indices.len() as u32;

        let vertices = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let indices = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        TextBuffers {
            vertices,
            indices,
            length,
        }
    }
}

pub fn character_quads_from_str(text: &str) -> Vec<CharacterQuad> {
    let CHARACTER_HASH: HashMap<char, u32> = HashMap::from(
        [
            ('A', 0),
            ('B', 1),
            ('C', 2),
            ('D', 3),
            ('E', 4),
            ('F', 5),
            ('G', 6),
            ('H', 7),
            ('I', 8),
            ('J', 9),
            ('K', 10),
            ('L', 11),
            ('M', 12),
            ('N', 13),
            ('O', 14),
            ('P', 15),
            ('Q', 16),
            ('R', 17),
            ('S', 18),
            ('T', 19),
            ('U', 20),
            ('V', 21),
            ('W', 22),
            ('X', 23),
            ('Y', 24),
            ('Z', 25),
            ('a', 26),
            ('b', 27),
            ('c', 28),
            ('d', 29),
            ('e', 30),
            ('f', 31),
            ('g', 32),
            ('h', 33),
            //('i', 34), //forgot i in spritesheet
            ('j', 34),
            ('k', 35),
            ('l', 36),
            ('m', 37),
            ('n', 38),
            ('o', 40), // messed up spritesheet here aswell
            ('p', 41),
            ('q', 42),
            ('r', 43),
            ('s', 44),
            ('t', 45),
            ('y', 46),
            ('v', 47),
            ('w', 48),
            ('x', 49),
            ('y', 50),
            ('z', 51),
            (' ', 52),
        ]
    );

    text.chars()
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
            let character = match CHARACTER_HASH.get(&c) {
                Some(v) => *v,
                None => 0,
            };
            let size = [0.1, 0.1];

            let position = [-0.5 + (0.1 * i as f32), 0.5, 0.0];

            CharacterQuad {
                position,
                size,
                character
            }
        })
        .collect()
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CharacterQuad {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub character: u32,
}

impl CharacterQuad {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CharacterVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                }
            ]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CharacterVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl CharacterVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CharacterVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}
