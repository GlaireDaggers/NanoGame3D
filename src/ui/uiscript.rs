use std::{fs, rc::Rc};

use fontdue::layout::{HorizontalAlign, LayoutSettings, VerticalAlign};
use ruwren::{create_module, get_slot_checked, Class, FunctionHandle, FunctionSignature, Handle, ModuleLibrary, ModuleScriptLoader, Printer, VMConfig, VMWrapper, VM};

use crate::{asset_loader::{load_font, load_material, load_texture, MaterialHandle, TextureHandle}, graphics::{buffer::Buffer, texture::Texture}, math::{Matrix4x4, Vector2, Vector3, Vector4}, misc::{Color32, Rectangle}};

use super::{font::FontPainter, ui_vertex::UiVertex};

struct WrenPrinter;

impl Printer for WrenPrinter {
    fn print(&mut self, s: String) {
        if s == "\n" {
            return;
        }

        log::info!("(SCRIPT) {}", s);
    }
}

struct WrenLoader;

impl ModuleScriptLoader for WrenLoader {
    fn load_script(&mut self, name: String) -> Option<String> {
        match fs::read_to_string(format!("content/scripts/{}.wren", name)) {
            Ok(v) => Some(v),
            Err(_) => None
        }
    }
}

struct ScriptTexture {
    texture: TextureHandle,
}

impl Class for ScriptTexture {
    fn initialize(vm: &VM) -> Self
    where
        Self: Sized {
        let path = get_slot_checked!(vm => string 1);
        
        ScriptTexture { texture: load_texture(path.as_str()).unwrap() }
    }
}

impl ScriptTexture {
    pub fn width(&self, vm: &VM) {
        vm.ensure_slots(1);
        vm.set_slot_double(0, self.texture.width() as f64);
    }

    pub fn height(&self, vm: &VM) {
        vm.ensure_slots(1);
        vm.set_slot_double(0, self.texture.height() as f64);
    }
}

struct ScriptFont {
    font: FontPainter,
}

impl Class for ScriptFont {
    fn initialize(vm: &VM) -> Self
    where
        Self: Sized {
        let path = get_slot_checked!(vm => string 1);
        
        let font = load_font(path.as_str()).unwrap();
        ScriptFont { font: FontPainter::new(&font) }
    }
}

impl ScriptFont {
    pub fn draw_text(&mut self, vm: &VM) {
        let painter = get_slot_checked!(vm => foreign_mut UiPainter => 1);
        let text = get_slot_checked!(vm => string 2);
        let size = get_slot_checked!(vm => num 3) as f32;
        let px = get_slot_checked!(vm => num 4) as f32;
        let py = get_slot_checked!(vm => num 5) as f32;
        let cr = get_slot_checked!(vm => num 6) as f32;
        let cg = get_slot_checked!(vm => num 7) as f32;
        let cb = get_slot_checked!(vm => num 8) as f32;
        let ca = get_slot_checked!(vm => num 9) as f32;
        let color = Color32::from_vec4(Vector4::new(cr, cg, cb, ca));

        let mut layout = LayoutSettings::default();
        layout.x = px;
        layout.y = py;

        self.font.draw_string(painter, text.as_str(), size, color, layout);
    }

    pub fn draw_text_layout(&mut self, vm: &VM) {
        let painter = get_slot_checked!(vm => foreign_mut UiPainter => 1);
        let text = get_slot_checked!(vm => string 2);
        let size = get_slot_checked!(vm => num 3) as f32;
        let px = get_slot_checked!(vm => num 4) as f32;
        let py = get_slot_checked!(vm => num 5) as f32;
        let w = get_slot_checked!(vm => num 6) as f32;
        let h = get_slot_checked!(vm => num 7) as f32;
        let halign = get_slot_checked!(vm => num 8) as i32;
        let valign = get_slot_checked!(vm => num 9) as i32;
        let cr = get_slot_checked!(vm => num 10) as f32;
        let cg = get_slot_checked!(vm => num 11) as f32;
        let cb = get_slot_checked!(vm => num 12) as f32;
        let ca = get_slot_checked!(vm => num 13) as f32;
        let color = Color32::from_vec4(Vector4::new(cr, cg, cb, ca));

        let mut layout = LayoutSettings::default();
        layout.x = px;
        layout.y = py;
        layout.max_width = Some(w);
        layout.max_height = Some(h);

        layout.horizontal_align = match halign {
            0 => HorizontalAlign::Left,
            1 => HorizontalAlign::Center,
            2 => HorizontalAlign::Right,
            _ => {
                vm.set_slot_string(0, "Invalid horizontal align");
                vm.abort_fiber(0);
                return;
            }
        };

        layout.vertical_align = match valign {
            0 => VerticalAlign::Top,
            1 => VerticalAlign::Middle,
            2 => VerticalAlign::Bottom,
            _ => {
                vm.set_slot_string(0, "Invalid vertical align");
                vm.abort_fiber(0);
                return;
            }
        };

        self.font.draw_string(painter, text.as_str(), size, color, layout);
    }
}

pub struct UiPainter {
    texture: Option<u32>,
    max_quads: usize,
    material: MaterialHandle,
    vertices: Vec<UiVertex>,
    indices: Vec<u16>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    window_size: (u32, u32),
}

impl Class for UiPainter {
    fn initialize(vm: &VM) -> Self
    where
        Self: Sized {
        let max_quads = get_slot_checked!(vm => num 1) as usize;

        UiPainter {
            texture: None,
            max_quads,
            material: load_material("content/materials/misc/ui.mat.ron").unwrap(),
            vertices: Vec::with_capacity(max_quads * 4),
            indices: Vec::with_capacity(max_quads * 6),
            vertex_buffer: Buffer::new((max_quads * 4 * size_of::<UiVertex>()) as isize),
            index_buffer: Buffer::new((max_quads * 6 * size_of::<u16>()) as isize),
            window_size: (0, 0),
        }
    }
}

impl UiPainter {
    fn flush_batch(self: &mut Self) {
        if self.vertices.len() == 0 {
            return;
        }

        let tex = self.texture.as_ref().unwrap();

        self.vertex_buffer.resize(self.vertex_buffer.size());
        self.index_buffer.resize(self.index_buffer.size());

        self.vertex_buffer.set_data(0, &self.vertices);
        self.index_buffer.set_data(0, &self.indices);

        let matrix = Matrix4x4::scale(Vector3::new(2.0 / self.window_size.0 as f32, -2.0 / self.window_size.1 as f32, 1.0))
            * Matrix4x4::translation(Vector3::new(-1.0, 1.0, 0.0));
        
        self.material.apply();
        self.material.shader.set_uniform_mat4("mvp", matrix);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, *tex);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            self.material.shader.inner.set_uniform_int("mainTexture", 0);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer.handle());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer.handle());

            UiVertex::setup_vtx_arrays(&self.material.shader);

            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const _);
        }

        self.vertices.clear();
        self.indices.clear();
        self.texture = None;
    }

    pub fn begin(&mut self, vm: &VM) {
        // width, height
        let window_width = get_slot_checked!(vm => num 1) as u32;
        let window_height = get_slot_checked!(vm => num 2) as u32;

        self.window_size = (window_width, window_height);
    }

    pub fn end(&mut self, _: &VM) {
        self.flush_batch();
    }

    pub fn draw_sprite_impl(&mut self, tex: &Texture, position: Vector2, size: Vector2, pivot: Vector2, rotation: f32, tex_rect: Rectangle, tint: Color32) {
        if let Some(prev_tex) = &self.texture {
            if *prev_tex != tex.handle() {
                // flush batch
                self.flush_batch();
            }
        }

        if self.vertices.len() >= self.max_quads * 4 {
            self.flush_batch();
        }

        self.texture = Some(tex.handle());

        let offset = pivot * size * -1.0;

        let pos_a = Vector2::new(0.0, 0.0) + offset;
        let pos_b = Vector2::new(size.x, 0.0) + offset;
        let pos_c = Vector2::new(0.0, size.y) + offset;
        let pos_d = Vector2::new(size.x, size.y) + offset;

        let pos_a = position + pos_a.rotate(rotation);
        let pos_b = position + pos_b.rotate(rotation);
        let pos_c = position + pos_c.rotate(rotation);
        let pos_d = position + pos_d.rotate(rotation);

        let uv_scale = 1.0 / Vector2::new(tex.width() as f32, tex.height() as f32);
        let uv_min = Vector2::new(tex_rect.x as f32, tex_rect.y as f32) * uv_scale;
        let uv_max = uv_min + (Vector2::new(tex_rect.w as f32, tex_rect.h as f32) * uv_scale);

        let uv_a = Vector2::new(uv_min.x, uv_min.y);
        let uv_b = Vector2::new(uv_max.x, uv_min.y);
        let uv_c = Vector2::new(uv_min.x, uv_max.y);
        let uv_d = Vector2::new(uv_max.x, uv_max.y);

        let idx_base = self.vertices.len() as u16;

        self.vertices.push(UiVertex { pos: pos_a, uv: uv_a, col: tint });
        self.vertices.push(UiVertex { pos: pos_b, uv: uv_b, col: tint });
        self.vertices.push(UiVertex { pos: pos_c, uv: uv_c, col: tint });
        self.vertices.push(UiVertex { pos: pos_d, uv: uv_d, col: tint });

        self.indices.push(idx_base);
        self.indices.push(idx_base + 1);
        self.indices.push(idx_base + 2);

        self.indices.push(idx_base + 1);
        self.indices.push(idx_base + 3);
        self.indices.push(idx_base + 2);
    }

    pub fn draw_sprite(&mut self, vm: &VM) {
        // texture
        // position x, y
        // size x, y
        // pivot x, y
        // rotation
        // tex rect x, y, w, h
        // tint r, g, b, a
        let tex = get_slot_checked!(vm => foreign ScriptTexture => 1);
        let px = get_slot_checked!(vm => num 2) as f32;
        let py = get_slot_checked!(vm => num 3) as f32;
        let sx = get_slot_checked!(vm => num 4) as f32;
        let sy = get_slot_checked!(vm => num 5) as f32;
        let ax = get_slot_checked!(vm => num 6) as f32;
        let ay = get_slot_checked!(vm => num 7) as f32;
        let rot = get_slot_checked!(vm => num 8) as f32;
        let tx = get_slot_checked!(vm => num 9) as i32;
        let ty = get_slot_checked!(vm => num 10) as i32;
        let tw = get_slot_checked!(vm => num 11) as i32;
        let th = get_slot_checked!(vm => num 12) as i32;
        let cr = get_slot_checked!(vm => num 13) as f32;
        let cg = get_slot_checked!(vm => num 14) as f32;
        let cb = get_slot_checked!(vm => num 15) as f32;
        let ca = get_slot_checked!(vm => num 16) as f32;

        let position = Vector2::new(px, py);
        let size = Vector2::new(sx, sy);
        let pivot = Vector2::new(ax, ay);
        let rot = rot.to_radians();
        let tex_rect = Rectangle::new(tx, ty, tw, th);
        let color = Color32::from_vec4(Vector4::new(cr, cg, cb, ca));

        self.draw_sprite_impl(&tex.texture, position, size, pivot, rot, tex_rect, color);
    }
}

create_module! {
    class("Painter") crate::ui::uiscript::UiPainter => painter {
        instance(fn "begin", 2) begin,
        instance(fn "end", 0) end,
        instance(fn "draw_sprite", 16) draw_sprite
    }

    class("Texture") crate::ui::uiscript::ScriptTexture => texture {
        instance(getter "width") width,
        instance(getter "height") height
    }

    class("Font") crate::ui::uiscript::ScriptFont => font {
        instance(fn "draw_text", 9) draw_text,
        instance(fn "draw_text", 13) draw_text_layout
    }

    module => engine
}

pub fn init_vm() -> VMWrapper {
    let mut lib = ModuleLibrary::new();
    engine::publish_module(&mut lib);

    VMConfig::new()
        .printer(WrenPrinter {})
        .script_loader(WrenLoader {})
        .enable_relative_import(true)
        .library(&lib)
        .build()
}

pub struct UiScript<'a> {
    update_fn: Rc<FunctionHandle<'a>>,
    paint_fn: Rc<FunctionHandle<'a>>,
    script_instance: Rc<Handle<'a>>,
}

impl<'a> UiScript<'a> {
    pub fn new(vm: &'a VMWrapper, module_name: &str, class_name: &str) -> UiScript<'a> {
        // load script
        let bootstrap = format!(
            r##"
            import "{module_name}" for {class_name}
            var script_instance = {class_name}.new()
            "##
        );
        vm.interpret("main", bootstrap).unwrap();

        // get handle to script instance
        let paint_fn = vm.make_call_handle(FunctionSignature::new_function("paint", 2));
        let update_fn = vm.make_call_handle(FunctionSignature::new_function("update", 1));
        let script_instance = {
            vm.execute(|vm| {
                vm.ensure_slots(1);
                vm.get_variable("main", "script_instance", 0);
            });
            vm.get_slot_handle(0)
        };

        UiScript {
            paint_fn,
            update_fn,
            script_instance,
        }
    }

    pub fn update(&self, vm: &'a VMWrapper, delta: f32) {
        vm.execute(|vm| {
            vm.ensure_slots(2);
            vm.set_slot_double(1, delta as f64);
        });

        vm.set_slot_handle(0, &self.script_instance);
        vm.call_handle(&self.update_fn).unwrap();
    }

    pub fn paint(&self, vm: &'a VMWrapper, window_size: (u32, u32)) {
        vm.execute(|vm| {
            vm.ensure_slots(3);
            vm.set_slot_double(1, window_size.0 as f64);
            vm.set_slot_double(2, window_size.1 as f64);
        });

        vm.set_slot_handle(0, &self.script_instance);
        vm.call_handle(&self.paint_fn).unwrap();
    }
}