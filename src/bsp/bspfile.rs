use std::{collections::HashMap, io::Seek};

use byteorder::{LittleEndian, ReadBytesExt};
use regex::Regex;

use crate::{math::{Vector2, Vector3, Vector4}, misc::Color32};

const BSP_MAGIC: u32 = 0x50534249;  // IBSP
const BSPX_MAGIC: u32 = 0x58505342; // BSPX
const BSP_VERSION: u32 = 38;

pub const SURF_LIGHT: u32   = 0x1;
pub const SURF_SLICK: u32   = 0x2;
pub const SURF_SKY: u32     = 0x4;
pub const SURF_WARP: u32    = 0x8;
pub const SURF_TRANS33: u32 = 0x10;
pub const SURF_TRANS66: u32 = 0x20;
pub const SURF_FLOW: u32    = 0x40;
pub const SURF_NODRAW: u32  = 0x80;

pub const SURF_NOLM: u32    = SURF_NODRAW | SURF_SKY | SURF_WARP | SURF_TRANS33 | SURF_TRANS66;

pub const CONTENTS_SOLID: u32       = 1;
pub const CONTENTS_WINDOW: u32      = 2;
pub const CONTENTS_AUX: u32         = 4;
pub const CONTENTS_LAVA: u32        = 8;
pub const CONTENTS_SLIME: u32       = 16;
pub const CONTENTS_WATER: u32       = 32;
pub const CONTENTS_MIST: u32        = 64;

pub const MASK_SOLID: u32           = CONTENTS_SOLID | CONTENTS_WINDOW;

fn read_vec2f<R: ReadBytesExt>(reader: &mut R) -> Vector2 {
    let x = reader.read_f32::<LittleEndian>().unwrap();
    let y = reader.read_f32::<LittleEndian>().unwrap();

    Vector2::new(x, y)
}

fn read_vec3f<R: ReadBytesExt>(reader: &mut R) -> Vector3 {
    let x = reader.read_f32::<LittleEndian>().unwrap();
    let y = reader.read_f32::<LittleEndian>().unwrap();
    let z = reader.read_f32::<LittleEndian>().unwrap();

    Vector3::new(x, y, z)
}

fn read_vec4f<R: ReadBytesExt>(reader: &mut R) -> Vector4 {
    let x = reader.read_f32::<LittleEndian>().unwrap();
    let y = reader.read_f32::<LittleEndian>().unwrap();
    let z = reader.read_f32::<LittleEndian>().unwrap();
    let w = reader.read_f32::<LittleEndian>().unwrap();

    Vector4::new(x, y, z, w)
}

fn read_vec3s<R: ReadBytesExt>(reader: &mut R) -> Vector3 {
    let x = reader.read_i16::<LittleEndian>().unwrap() as f32;
    let y = reader.read_i16::<LittleEndian>().unwrap() as f32;
    let z = reader.read_i16::<LittleEndian>().unwrap() as f32;

    Vector3::new(x, y, z)
}

fn read_vec3i<R: ReadBytesExt>(reader: &mut R) -> Vector3 {
    let x = reader.read_i32::<LittleEndian>().unwrap() as f32;
    let y = reader.read_i32::<LittleEndian>().unwrap() as f32;
    let z = reader.read_i32::<LittleEndian>().unwrap() as f32;

    Vector3::new(x, y, z)
}

fn read_color24<R: ReadBytesExt>(reader: &mut R) -> Color32 {
    let r = reader.read_u8().unwrap();
    let g = reader.read_u8().unwrap();
    let b = reader.read_u8().unwrap();

    Color32::new(r, g, b, 255)
}

fn read_color32<R: ReadBytesExt>(reader: &mut R) -> Color32 {
    let r = reader.read_u8().unwrap();
    let g = reader.read_u8().unwrap();
    let b = reader.read_u8().unwrap();
    let a = reader.read_u8().unwrap();

    Color32::new(r, g, b, a)
}

pub struct BspLumpInfo {
    offset: u32,
    length: u32,
}

#[derive(Clone, Copy)]
pub struct Edge {
    pub a: u16,
    pub b: u16
}

pub struct BspFace {
    pub _plane: u16,
    pub _plane_side: u16,
    pub first_edge: u32,
    pub num_edges: u16,
    pub texture_info: u16,
    pub lightmap_styles: [u8;4],
    pub num_lightmaps: usize,
    pub lightmap_offset: u32,
}

pub struct Plane {
    pub normal: Vector3,
    pub distance: f32,
    pub plane_type: u32,
}

pub struct Node {
    pub plane: u32,
    pub front_child: i32,
    pub back_child: i32,
    pub _bbox_min: Vector3,
    pub _bbox_max: Vector3,
    pub _first_face: u16,
    pub _num_faces: u16,
}

pub struct Leaf {
    pub contents: u32,
    pub cluster: u16,
    pub _area: u16,
    pub bbox_min: Vector3,
    pub bbox_max: Vector3,
    pub first_leaf_face: u16,
    pub num_leaf_faces: u16,
    pub first_leaf_brush: u16,
    pub num_leaf_brushes: u16
}

pub struct TexInfo {
    pub u_axis: Vector3,
    pub u_offset: f32,
    pub v_axis: Vector3,
    pub v_offset: f32,
    pub flags: u32,
    pub _value: u32,
    pub texture_name: String,
    pub _next_texinfo: u32,
}

pub struct Brush {
    pub first_brush_side: u32,
    pub num_brush_sides: u32,
    pub contents: u32,
}

pub struct BrushSide {
    pub plane: u16,
    pub _tex: u16,
}

pub struct VisCluster {
    pub vis_offset: usize
}

pub struct SubModel {
    pub mins: Vector3,
    pub maxs: Vector3,
    pub origin: Vector3,
    pub headnode: u32,
    pub first_face: u32,
    pub num_faces: u32,
}

pub struct EntityLump {
    pub entities: String
}

pub struct VertexLump {
    pub vertices: Vec<Vector3>
}

pub struct EdgeLump {
    pub edges: Vec<Edge>
}

pub struct FaceLump {
    pub faces: Vec<BspFace>
}

pub struct FaceEdgeLump {
    pub edges: Vec<i32>
}

pub struct PlaneLump {
    pub planes: Vec<Plane>
}

pub struct NodeLump {
    pub nodes: Vec<Node>
}

pub struct LeafLump {
    pub leaves: Vec<Leaf>
}

pub struct LeafFaceLump {
    pub faces: Vec<u16>
}

pub struct LeafBrushLump {
    pub brushes: Vec<u16>
}

pub struct TexInfoLump {
    pub textures: Vec<TexInfo>
}

pub struct VisLump {
    pub clusters: Vec<VisCluster>,
    pub vis_buffer: Vec<u8>,
}

pub struct BrushLump {
    pub brushes: Vec<Brush>
}

pub struct BrushSideLump {
    pub brush_sides: Vec<BrushSide>
}

pub struct SubModelLump {
    pub submodels: Vec<SubModel>
}

pub struct LightmapLump {
    pub lm: Vec<Color32>
}

#[derive(Clone, Copy)]
pub struct LSHProbe {
    pub sh_r: Vector4,
    pub sh_g: Vector4,
    pub sh_b: Vector4,
}

impl LSHProbe {
    pub fn lerp(self: &Self, b: Self, t: f32) -> Self {
        LSHProbe {
            sh_r: (self.sh_r * (1.0 - t)) + (b.sh_r * t),
            sh_g: (self.sh_g * (1.0 - t)) + (b.sh_g * t),
            sh_b: (self.sh_b * (1.0 - t)) + (b.sh_b * t)
        }
    }

    pub fn sample(self: &Self, direction: Vector3) -> Vector3 {
        let v = Vector4::new(direction.x, direction.y, direction.z, 1.0);
        let r = v.dot(self.sh_r);
        let g = v.dot(self.sh_g);
        let b = v.dot(self.sh_b);

        Vector3::new(r, g, b)
    }
}

pub struct LSHGridLump {
    pub grid_dist: Vector3,
    pub grid_size: Vector3,
    pub grid_mins: Vector3,
    pub probes: Vec<LSHProbe>
}

pub struct StaticProp {
    pub material: u32,
    pub topology: gl::types::GLenum,
    pub first_index: u32,
    pub num_indices: u32,
    pub first_vertex: u32,
    pub num_vertices: u32,
}

pub struct StaticPropLump {
    pub props: Vec<StaticProp>
}

pub struct StaticPropIndicesLump {
    pub indices: Vec<u16>
}

pub struct StaticPropVertex {
    pub position: Vector4,
    pub normal: Vector4,
    pub tangent: Vector4,
    pub texcoord: Vector2,
    pub color: Color32,
}

pub struct StaticPropVerticesLump {
    pub vertices: Vec<StaticPropVertex>
}

pub struct StaticPropMaterialsLump {
    pub materials: Vec<String>
}

impl EntityLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> EntityLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let mut data: Vec<u8> = vec![0;info.length as usize];
        reader.read_exact(&mut data).unwrap();

        let mut len = 0;
        for val in &data {
            if *val == 0 {
                break;
            }
            len += 1;
        }

        let slice = &data[0..len];
        let entities = unsafe { std::str::from_utf8_unchecked(slice).to_owned() };

        EntityLump {
            entities
        }
    }

    pub fn parse<F>(self: &Self, mut f: F) where F: FnMut(HashMap<&str, &str>) {
        let re = Regex::new("(\"(.*)\"[ \t]+\"(.*)\")").unwrap();

        // find ranges of data between { and }
        let mut slices = Vec::new();
        let mut start = 0;
        for (idx, v) in self.entities.as_bytes().iter().enumerate() {
            if *v == b'{' {
                start = idx + 1;
            }
            else if *v == b'}' {
                slices.push((start, idx - 1));
            }
        }

        // parse key value pairs
        for (start, end) in slices {
            let entitydata = &self.entities[start..end];
            
            let mut map = HashMap::new();
            for (_, [_, propname, propval]) in re.captures_iter(entitydata).map(|c| c.extract()) {
                map.insert(propname, propval);
            }

            f(map);
        }
    }
}

impl VertexLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> VertexLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_vertices = (info.length / 12) as usize;
        let mut vertices: Vec<Vector3> = Vec::with_capacity(num_vertices);

        for _ in 0..num_vertices {
            vertices.push(read_vec3f(reader));
        }

        VertexLump {
            vertices
        }
    }
}

impl EdgeLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> EdgeLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_edges = (info.length / 4) as usize;
        let mut edges: Vec<Edge> = Vec::with_capacity(num_edges);

        for _ in 0..num_edges {
            let a = reader.read_u16::<LittleEndian>().unwrap();
            let b = reader.read_u16::<LittleEndian>().unwrap();
            edges.push(Edge {a, b});
        }

        EdgeLump {
            edges
        }
    }
}

impl FaceLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> FaceLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_faces = (info.length / 20) as usize;
        let mut faces: Vec<BspFace> = Vec::with_capacity(num_faces);

        for _ in 0..num_faces {
            let plane = reader.read_u16::<LittleEndian>().unwrap();
            let plane_side = reader.read_u16::<LittleEndian>().unwrap();
            let first_edge = reader.read_u32::<LittleEndian>().unwrap();
            let num_edges = reader.read_u16::<LittleEndian>().unwrap();
            let texture_info = reader.read_u16::<LittleEndian>().unwrap();
            let lightmap_styles = [
                reader.read_u8().unwrap(),
                reader.read_u8().unwrap(),
                reader.read_u8().unwrap(),
                reader.read_u8().unwrap()
            ];
            let lightmap_offset = reader.read_u32::<LittleEndian>().unwrap();

            let mut num_lightmaps = 0;

            for ls in &lightmap_styles {
                if *ls == 255 {
                    break;
                }
                num_lightmaps += 1;
            }

            faces.push(BspFace {
                _plane: plane, _plane_side: plane_side, first_edge, num_edges, texture_info, lightmap_styles, num_lightmaps, lightmap_offset
            });
        }

        FaceLump {
            faces
        }
    }
}

impl FaceEdgeLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> FaceEdgeLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_edges = (info.length / 4) as usize;
        let mut edges: Vec<i32> = Vec::with_capacity(num_edges);

        for _ in 0..num_edges {
            edges.push(reader.read_i32::<LittleEndian>().unwrap());
        }

        FaceEdgeLump {
            edges
        }
    }
}

impl PlaneLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> PlaneLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_planes = (info.length / 20) as usize;
        let mut planes: Vec<Plane> = Vec::with_capacity(num_planes);

        for _ in 0..num_planes {
            let normal = read_vec3f(reader);
            let distance = reader.read_f32::<LittleEndian>().unwrap();
            let plane_type = reader.read_u32::<LittleEndian>().unwrap();
            planes.push(Plane { normal, distance, plane_type });
        }

        PlaneLump {
            planes
        }
    }
}

impl NodeLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> NodeLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_nodes = (info.length / 28) as usize;
        let mut nodes: Vec<Node> = Vec::with_capacity(num_nodes);

        println!("Num nodes in node lump: {}", num_nodes);

        for _ in 0..num_nodes {
            let plane = reader.read_u32::<LittleEndian>().unwrap();
            let front_child = reader.read_i32::<LittleEndian>().unwrap();
            let back_child = reader.read_i32::<LittleEndian>().unwrap();
            let bbox_min = read_vec3s(reader);
            let bbox_max = read_vec3s(reader);
            let first_face = reader.read_u16::<LittleEndian>().unwrap();
            let num_faces = reader.read_u16::<LittleEndian>().unwrap();

            nodes.push(Node {
                plane,
                front_child,
                back_child,
                _bbox_min: bbox_min,
                _bbox_max: bbox_max,
                _first_face: first_face,
                _num_faces: num_faces
            });
        }

        NodeLump {
            nodes
        }
    }
}

impl LeafLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> LeafLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_leaves = (info.length / 28) as usize;
        let mut leaves: Vec<Leaf> = Vec::with_capacity(num_leaves);

        println!("Num leaves in leaf lump: {}", num_leaves);

        for _ in 0..num_leaves {
            let brush_or = reader.read_u32::<LittleEndian>().unwrap();
            let cluster = reader.read_u16::<LittleEndian>().unwrap();
            let area = reader.read_u16::<LittleEndian>().unwrap();
            let bbox_min = read_vec3s(reader);
            let bbox_max = read_vec3s(reader);
            let first_leaf_face = reader.read_u16::<LittleEndian>().unwrap();
            let num_leaf_faces = reader.read_u16::<LittleEndian>().unwrap();
            let first_leaf_brush = reader.read_u16::<LittleEndian>().unwrap();
            let num_leaf_brushes = reader.read_u16::<LittleEndian>().unwrap();

            leaves.push(Leaf {
                contents: brush_or,
                cluster,
                _area: area,
                bbox_min,
                bbox_max,
                first_leaf_face,
                num_leaf_faces,
                first_leaf_brush,
                num_leaf_brushes
            });
        }

        LeafLump {
            leaves
        }
    }
}

impl LeafFaceLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> LeafFaceLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_faces = (info.length / 2) as usize;
        let mut faces: Vec<u16> = Vec::with_capacity(num_faces);

        for _ in 0..num_faces {
            let a = reader.read_u16::<LittleEndian>().unwrap();
            faces.push(a);
        }

        LeafFaceLump {
            faces
        }
    }
}

impl LeafBrushLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> LeafBrushLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_brushes = (info.length / 2) as usize;
        let mut brushes: Vec<u16> = Vec::with_capacity(num_brushes);

        for _ in 0..num_brushes {
            let a = reader.read_u16::<LittleEndian>().unwrap();
            brushes.push(a);
        }

        LeafBrushLump {
            brushes
        }
    }
}

impl TexInfoLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> TexInfoLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_textures = (info.length / 76) as usize;
        let mut textures: Vec<TexInfo> = Vec::with_capacity(num_textures);

        println!("Num textures in tex info lump: {}", num_textures);

        for _ in 0..num_textures {
            let u_axis = read_vec3f(reader);
            let u_offset = reader.read_f32::<LittleEndian>().unwrap();

            let v_axis = read_vec3f(reader);
            let v_offset = reader.read_f32::<LittleEndian>().unwrap();

            let flags = reader.read_u32::<LittleEndian>().unwrap();
            let value = reader.read_u32::<LittleEndian>().unwrap();

            let mut texture_name: [u8; 32] = [0; 32];
            reader.read_exact(&mut texture_name).unwrap();

            let mut name_len = 32;
            for i in 0..32 {
                if texture_name[i] == 0 {
                    name_len = i;
                    break;
                }
            }

            let texture_name = unsafe { std::str::from_utf8_unchecked(&texture_name[0..name_len]) }.to_owned();
            let next_texinfo = reader.read_u32::<LittleEndian>().unwrap();

            textures.push(TexInfo {
                u_axis,
                u_offset,
                v_axis,
                v_offset,
                flags,
                _value: value,
                texture_name,
                _next_texinfo: next_texinfo,
            });
        }

        TexInfoLump {
            textures
        }
    }
}

impl VisLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> VisLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_clusters = reader.read_u32::<LittleEndian>().unwrap() as usize;
        let hdr_size = 4 + (num_clusters * 8);

        let mut clusters: Vec<VisCluster> = Vec::with_capacity(num_clusters);

        println!("Num clusters in vis lump: {}", num_clusters);

        for _ in 0..num_clusters {
            let pvs = reader.read_u32::<LittleEndian>().unwrap();
            let _phs = reader.read_u32::<LittleEndian>().unwrap();

            let offs = (pvs as usize) - hdr_size;

            clusters.push(VisCluster {
                vis_offset: offs
            });
        }

        // read remainder of lump as byte array
        let buf_len = (info.length as usize) - hdr_size;
        let mut vis_buffer: Vec<u8> = vec![0;buf_len];
        reader.read_exact(&mut vis_buffer).unwrap();

        VisLump {
            clusters,
            vis_buffer
        }
    }

    // Unpack vis info for a given cluster index
    pub fn unpack_vis(self: &VisLump, cluster_index: usize, vis_info: &mut [bool]) {
        let mut v = self.clusters[cluster_index].vis_offset;
        let mut c = 0;

        while c < self.clusters.len() {
            if self.vis_buffer[v] == 0 {
                v += 1;
                c += 8 * (self.vis_buffer[v] as usize);
            }
            else {
                for bit in 0..8 {
                    let m = 1 << bit;
                    if (self.vis_buffer[v] & m) != 0 {
                        vis_info[c] = true;
                    }
                    c += 1;
                }
            }

            v += 1;
        }
    }
}

impl LightmapLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> LightmapLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_px = (info.length / 3) as usize;
        let mut lm: Vec<Color32> = Vec::with_capacity(num_px);

        for _ in 0..num_px {
            let mut c = read_color24(reader);
            c.r = (c.r as i32 * 2).clamp(0, 255) as u8;
            c.g = (c.g as i32 * 2).clamp(0, 255) as u8;
            c.b = (c.b as i32 * 2).clamp(0, 255) as u8;
            lm.push(c);
        }

        LightmapLump {
            lm
        }
    }
}

impl BrushLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> BrushLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_brushes = (info.length / 12) as usize;
        let mut brushes: Vec<Brush> = Vec::with_capacity(num_brushes);

        for _ in 0..num_brushes {
            let first_brush_side = reader.read_u32::<LittleEndian>().unwrap();
            let num_brush_sides = reader.read_u32::<LittleEndian>().unwrap();
            let contents = reader.read_u32::<LittleEndian>().unwrap();

            brushes.push(Brush { first_brush_side, num_brush_sides, contents });
        }

        BrushLump {
            brushes
        }
    }
}

impl BrushSideLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> BrushSideLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_brush_sides = (info.length / 4) as usize;
        let mut brush_sides: Vec<BrushSide> = Vec::with_capacity(num_brush_sides);

        for _ in 0..num_brush_sides {
            let plane = reader.read_u16::<LittleEndian>().unwrap();
            let tex = reader.read_u16::<LittleEndian>().unwrap();

            brush_sides.push(BrushSide { plane, _tex: tex });
        }

        BrushSideLump {
            brush_sides
        }
    }
}

impl SubModelLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> SubModelLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let num_submodels = (info.length / 48) as usize;
        let mut submodels: Vec<SubModel> = Vec::with_capacity(num_submodels);

        for _ in 0..num_submodels {
            let mins = read_vec3f(reader);
            let maxs = read_vec3f(reader);
            let origin = read_vec3f(reader);

            let headnode = reader.read_u32::<LittleEndian>().unwrap();
            let first_face = reader.read_u32::<LittleEndian>().unwrap();
            let num_faces = reader.read_u32::<LittleEndian>().unwrap();

            submodels.push(SubModel {
                mins,
                maxs,
                origin,
                headnode,
                first_face,
                num_faces
            });
        }

        SubModelLump {
            submodels
        }
    }
}

impl LSHGridLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> LSHGridLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let grid_dist = read_vec3f(reader);
        let grid_size = read_vec3i(reader);
        let grid_mins = read_vec3f(reader);

        let num_x = grid_size.x as i32;
        let num_y = grid_size.y as i32;
        let num_z = grid_size.z as i32;

        println!("LSH Grid: {} x {} x {}", num_x, num_y, num_z);

        let num_total = num_x * num_y * num_z;
        let mut probes = Vec::with_capacity(num_total as usize);

        for _ in 0..num_total {
            let l0_rgb = read_vec3f(reader);
            let l1_r = read_vec3f(reader);
            let l1_g = read_vec3f(reader);
            let l1_b = read_vec3f(reader);

            // TODO: x and y seem flipped?

            probes.push(LSHProbe {
                sh_r: Vector4::new(-l1_r.x, -l1_r.y, l1_r.z, l0_rgb.x),
                sh_g: Vector4::new(-l1_g.x, -l1_g.y, l1_g.z, l0_rgb.y),
                sh_b: Vector4::new(-l1_b.x, -l1_b.y, l1_b.z, l0_rgb.z),
            });
        }

        LSHGridLump { grid_dist, grid_size, grid_mins, probes }
    }

    pub fn sample_position(self: &Self, pos: Vector3) -> LSHProbe {
        let mut coord = (pos - self.grid_mins) / self.grid_dist;
        coord.x = coord.x.clamp(0.0, self.grid_size.x - 1.001);
        coord.y = coord.y.clamp(0.0, self.grid_size.y - 1.001);
        coord.z = coord.z.clamp(0.0, self.grid_size.z - 1.001);

        // gather a set of 8 samples surrounding the given point & perform trilinear interpolation to arrive at final SH probe

        let sx = self.grid_size.x as i32;
        let sy = self.grid_size.y as i32;

        let cx1 = coord.x as i32;
        let cy1 = coord.y as i32;
        let cz1 = coord.z as i32;

        let cx2 = cx1 + 1;
        let cy2 = cy1 + 1;
        let cz2 = cz1 + 1;

        let fracx = coord.x - cx1 as f32;
        let fracy = coord.y - cy1 as f32;
        let fracz = coord.z - cz1 as f32;

        let idx000 = cx1 + (cy1 * sx) + (cz1 * sx * sy);
        let idx100 = cx2 + (cy1 * sx) + (cz1 * sx * sy);
        let idx010 = cx1 + (cy2 * sx) + (cz1 * sx * sy);
        let idx110 = cx2 + (cy2 * sx) + (cz1 * sx * sy);
        let idx001 = cx1 + (cy1 * sx) + (cz2 * sx * sy);
        let idx101 = cx2 + (cy1 * sx) + (cz2 * sx * sy);
        let idx011 = cx1 + (cy2 * sx) + (cz2 * sx * sy);
        let idx111 = cx2 + (cy2 * sx) + (cz2 * sx * sy);

        let sh000 = self.probes[idx000 as usize];
        let sh100 = self.probes[idx100 as usize];
        let sh010 = self.probes[idx010 as usize];
        let sh110 = self.probes[idx110 as usize];
        let sh001 = self.probes[idx001 as usize];
        let sh101 = self.probes[idx101 as usize];
        let sh011 = self.probes[idx011 as usize];
        let sh111 = self.probes[idx111 as usize];

        // interpolate on X axis
        let shx00 = sh000.lerp(sh100, fracx);
        let shx10 = sh010.lerp(sh110, fracx);
        let shx01 = sh001.lerp(sh101, fracx);
        let shx11 = sh011.lerp(sh111, fracx);

        // interpolate on Y axis
        let shxy0 = shx00.lerp(shx10, fracy);
        let shxy1 = shx01.lerp(shx11, fracy);

        // interpolate on Z axis
        let shxyz = shxy0.lerp(shxy1, fracz);

        shxyz
    }
}

impl StaticPropLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> StaticPropLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let prop_count = reader.read_u32::<LittleEndian>().unwrap();

        let mut props = Vec::new();
        for _ in 0..prop_count {
            let material = reader.read_u32::<LittleEndian>().unwrap();
            let mode = reader.read_u32::<LittleEndian>().unwrap();
            let first_index = reader.read_u32::<LittleEndian>().unwrap();
            let num_indices = reader.read_u32::<LittleEndian>().unwrap();
            let first_vertex = reader.read_u32::<LittleEndian>().unwrap();
            let num_vertices = reader.read_u32::<LittleEndian>().unwrap();

            let topology = match mode {
                0 => {
                    gl::TRIANGLES
                },
                1 => {
                    gl::TRIANGLE_STRIP
                },
                _ => {
                    panic!("Invalid static prop topology")
                }
            };

            props.push(StaticProp { material, topology, first_index, num_indices, first_vertex, num_vertices });
        }

        StaticPropLump { props }
    }
}

impl StaticPropIndicesLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> StaticPropIndicesLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let index_count = reader.read_u32::<LittleEndian>().unwrap();

        let mut indices = Vec::new();
        for _ in 0..index_count {
            indices.push(reader.read_u16::<LittleEndian>().unwrap());
        }

        StaticPropIndicesLump { indices }
    }
}

impl StaticPropVerticesLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> StaticPropVerticesLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let vertex_count = reader.read_u32::<LittleEndian>().unwrap();

        let mut vertices = Vec::new();
        for _ in 0..vertex_count {
            let position = read_vec3f(reader);
            let normal = read_vec3f(reader);
            let tangent = read_vec4f(reader);
            let texcoord = read_vec2f(reader);
            let color = read_color32(reader);

            vertices.push(StaticPropVertex { 
                position: Vector4::new(position.x, position.y, position.z, 1.0),
                normal: Vector4::new(normal.x, normal.y, normal.z, 1.0),
                tangent,
                texcoord,
                color
            });
        }

        StaticPropVerticesLump { vertices }
    }
}

impl StaticPropMaterialsLump {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R, info: &BspLumpInfo) -> StaticPropMaterialsLump {
        reader.seek(std::io::SeekFrom::Start(info.offset as u64)).unwrap();

        let material_count = reader.read_u32::<LittleEndian>().unwrap();

        let mut materials = Vec::new();
        for _ in 0..material_count {
            let mut material_name: [u8; 64] = [0; 64];
            reader.read_exact(&mut material_name).unwrap();

            let mut name_len = 64;
            for i in 0..64 {
                if material_name[i] == 0 {
                    name_len = i;
                    break;
                }
            }

            let material_name = unsafe { std::str::from_utf8_unchecked(&material_name[0..name_len]) }.to_owned();
            materials.push(material_name);
        }

        StaticPropMaterialsLump { materials }
    }
}

pub struct BspFile {
    pub entity_lump: EntityLump,
    pub vertex_lump: VertexLump,
    pub edge_lump: EdgeLump,
    pub face_lump: FaceLump,
    pub face_edge_lump: FaceEdgeLump,
    pub plane_lump: PlaneLump,
    pub node_lump: NodeLump,
    pub leaf_lump: LeafLump,
    pub leaf_face_lump: LeafFaceLump,
    pub leaf_brush_lump: LeafBrushLump,
    pub tex_info_lump: TexInfoLump,
    pub vis_lump: VisLump,
    pub lm_lump: LightmapLump,
    pub brush_lump: BrushLump,
    pub brush_side_lump: BrushSideLump,
    pub submodel_lump: SubModelLump,
    pub lsh_grid_lump: LSHGridLump,
    pub sprop_lump: StaticPropLump,
    pub sprop_indices_lump: StaticPropIndicesLump,
    pub sprop_vertices_lump: StaticPropVerticesLump,
    pub sprop_materials_lump: StaticPropMaterialsLump,
}

impl BspFile {
    pub fn new<R: Seek + ReadBytesExt>(reader: &mut R) -> BspFile {
        let magic = reader.read_u32::<LittleEndian>().unwrap();
        if magic != BSP_MAGIC {
            panic!("Failed loading BSP: input is not valid IBSP data");
        }

        let version = reader.read_u32::<LittleEndian>().unwrap();
        if version != BSP_VERSION {
            panic!("Failed loading BSP: wrong IBSP file version");
        }

        // read BSP lump info
        let mut bsp_lumps: Vec<BspLumpInfo> = Vec::with_capacity(19);

        let mut max_offset = 0;

        for _ in 0..19 {
            let offset = reader.read_u32::<LittleEndian>().unwrap();
            let length = reader.read_u32::<LittleEndian>().unwrap();

            max_offset = max_offset.max((offset + length) as u64);

            bsp_lumps.push(BspLumpInfo { offset, length });
        }

        // read lumps
        let entity_lump = EntityLump::new(reader, &bsp_lumps[0]);
        let plane_lump = PlaneLump::new(reader, &bsp_lumps[1]);
        let vertex_lump = VertexLump::new(reader, &bsp_lumps[2]);
        let vis_lump = VisLump::new(reader, &bsp_lumps[3]);
        let node_lump = NodeLump::new(reader, &bsp_lumps[4]);
        let tex_info_lump = TexInfoLump::new(reader, &bsp_lumps[5]);
        let face_lump = FaceLump::new(reader, &bsp_lumps[6]);
        let lm_lump = LightmapLump::new(reader, &bsp_lumps[7]);
        let leaf_lump = LeafLump::new(reader, &bsp_lumps[8]);
        let leaf_face_lump = LeafFaceLump::new(reader, &bsp_lumps[9]);
        let leaf_brush_lump = LeafBrushLump::new(reader, &bsp_lumps[10]);
        let edge_lump = EdgeLump::new(reader, &bsp_lumps[11]);
        let face_edge_lump = FaceEdgeLump::new(reader, &bsp_lumps[12]);
        let submodel_lump = SubModelLump::new(reader, &bsp_lumps[13]);
        let brush_lump = BrushLump::new(reader, &bsp_lumps[14]);
        let brush_side_lump = BrushSideLump::new(reader, &bsp_lumps[15]);

        // seek to end of main BSP and look for "BSPX" header
        let bspx_start = if max_offset % 4 == 0 { max_offset } else { max_offset + (4 - max_offset % 4) };
        reader.seek(std::io::SeekFrom::Start(bspx_start)).unwrap();
        let bspx_magic = reader.read_u32::<LittleEndian>().unwrap();
        if bspx_magic != BSPX_MAGIC {
            panic!("Failed loading BSP: expected BSPX extension");
        }

        let num_bspx_lumps = reader.read_u32::<LittleEndian>().unwrap();
        let mut bspx_lumps = HashMap::new();

        for _ in 0..num_bspx_lumps {
            let mut lump_name: [u8; 24] = [0; 24];
            reader.read_exact(&mut lump_name).unwrap();

            let mut name_len = 32;
            for i in 0..32 {
                if lump_name[i] == 0 {
                    name_len = i;
                    break;
                }
            }

            let lump_name = unsafe { std::str::from_utf8_unchecked(&lump_name[0..name_len]) }.to_owned();
            let lump_offset = reader.read_u32::<LittleEndian>().unwrap();
            let lump_length = reader.read_u32::<LittleEndian>().unwrap();

            println!("BSPX Lump: {} (offs: {}, len: {})", lump_name, lump_offset, lump_length);

            let lump_info = BspLumpInfo { offset: lump_offset, length: lump_length };
            bspx_lumps.insert(lump_name, lump_info);
        }

        let lsh_grid_lump = LSHGridLump::new(reader, &bspx_lumps["LSH_GRID"]);
        let sprop_lump = StaticPropLump::new(reader, &bspx_lumps["SPROP"]);
        let sprop_indices_lump = StaticPropIndicesLump::new(reader, &bspx_lumps["SPROP_INDICES"]);
        let sprop_vertices_lump = StaticPropVerticesLump::new(reader, &bspx_lumps["SPROP_VERTICES"]);
        let sprop_materials_lump = StaticPropMaterialsLump::new(reader, &bspx_lumps["SPROP_MATERIALS"]);

        BspFile {
            entity_lump,
            vertex_lump,
            edge_lump,
            face_lump,
            face_edge_lump,
            plane_lump,
            node_lump,
            leaf_lump,
            leaf_face_lump,
            leaf_brush_lump,
            tex_info_lump,
            vis_lump,
            lm_lump,
            brush_lump,
            brush_side_lump,
            submodel_lump,
            lsh_grid_lump,
            sprop_lump,
            sprop_indices_lump,
            sprop_vertices_lump,
            sprop_materials_lump,
        }
    }
}