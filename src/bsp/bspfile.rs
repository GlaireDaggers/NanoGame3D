use std::{collections::HashMap, io::Seek};

use byteorder::{LittleEndian, ReadBytesExt};
use regex::Regex;

use crate::{math::Vector3, misc::Color32};

const BSP_MAGIC: u32 = 0x50534249;
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

fn read_vec3f<R: ReadBytesExt>(reader: &mut R) -> Vector3 {
    let x = reader.read_f32::<LittleEndian>().unwrap();
    let y = reader.read_f32::<LittleEndian>().unwrap();
    let z = reader.read_f32::<LittleEndian>().unwrap();

    Vector3::new(x, y, z)
}

fn read_vec3s<R: ReadBytesExt>(reader: &mut R) -> Vector3 {
    let x = reader.read_i16::<LittleEndian>().unwrap() as f32;
    let y = reader.read_i16::<LittleEndian>().unwrap() as f32;
    let z = reader.read_i16::<LittleEndian>().unwrap() as f32;

    Vector3::new(x, y, z)
}

fn read_color24<R: ReadBytesExt>(reader: &mut R) -> Color32 {
    let r = reader.read_u8().unwrap();
    let g = reader.read_u8().unwrap();
    let b = reader.read_u8().unwrap();

    Color32::new(r, g, b, 255)
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

        for _ in 0..19 {
            let offset = reader.read_u32::<LittleEndian>().unwrap();
            let length = reader.read_u32::<LittleEndian>().unwrap();

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
            submodel_lump
        }
    }
}