use std::sync::Arc;

use fontdue::layout::{HorizontalAlign, LayoutSettings, VerticalAlign, WrapStyle};
use rune::{runtime::{Function, InstAddress, Memory, Output, VmResult}, termcolor::{ColorChoice, StandardStream}, vm_try, Any, Context, ContextError, Diagnostics, FromValue, Module, Source, Sources, Value, Vm};

use crate::{asset_loader::{load_font, load_texture, TextureHandle}, math::Vector2, misc::{Color32, Rectangle}};

use super::{font::FontPainter, painter::UiPainter};

#[derive(Any)]
struct Texture {
    #[allow(unused)]
    texture: TextureHandle,
}

impl Texture {
    #[rune::function(path = Self::load)]
    pub fn load(path: &str) -> Texture {
        let texture = load_texture(path).unwrap();
        Texture { texture }
    }

    #[rune::function(instance)]
    pub fn width(&self) -> i32 {
        self.texture.width()
    }

    #[rune::function(instance)]
    pub fn height(&self) -> i32 {
        self.texture.height()
    }

    pub fn register_script(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<Self>()?;
        module.function_meta(Self::load)?;
        module.function_meta(Self::width)?;
        module.function_meta(Self::height)?;

        Ok(())
    }
}

#[derive(Any)]
enum HAlign {
    #[rune(constructor)]
    Left,
    #[rune(constructor)]
    Middle,
    #[rune(constructor)]
    Right,
}

#[derive(Any)]
enum VAlign {
    #[rune(constructor)]
    Top,
    #[rune(constructor)]
    Middle,
    #[rune(constructor)]
    Bottom,
}

#[derive(Any)]
enum TextWrap {
    #[rune(constructor)]
    Word,
    #[rune(constructor)]
    Letter,
}

#[derive(Any)]
struct Font {
    #[allow(unused)]
    font: FontPainter,
}

impl Font {
    #[rune::function(path = Font::load)]
    pub fn load(path: &str) -> Font {
        let font = load_font(path).unwrap();
        Font { font: FontPainter::new(&font) }
    }

    pub fn draw_text(&mut self, painter: &mut Painter, text: &str, size: f32, position: Vector2, color: Color32) {
        self.font.draw_string(&mut painter.painter, text, size, position, Vector2::zero(), 0.0, color, LayoutSettings::default());
    }

    pub fn draw_text_layout(&mut self, painter: &mut Painter,
        text: &str,
        size: f32,
        position: Vector2,
        pivot: Vector2,
        rotation: f32,
        width: f32,
        height: f32,
        h_align: HorizontalAlign,
        v_align: VerticalAlign,
        wrap: WrapStyle,
        color: Color32)
    {
        let mut layout = LayoutSettings::default();
        layout.max_width = Some(width);
        layout.max_height = Some(height);
        layout.wrap_style = wrap;
        layout.horizontal_align = h_align;
        layout.vertical_align = v_align;

        self.font.draw_string(&mut painter.painter, text, size, position, pivot * size, rotation, color, layout);
    }

    fn draw_text_wrapper(stack: &mut dyn Memory, addr: InstAddress, args: usize, _: Output) -> VmResult<()> {
        let args = vm_try!(stack.slice_at(addr, args));

        let mut this = vm_try!(args[0].borrow_mut::<Self>());
        let mut painter = vm_try!(args[1].borrow_mut::<Painter>());
        let text: String = vm_try!(String::from_value(args[2].clone()));
        let size = vm_try!(args[3].as_float());
        let position = vm_try!(Vector2::from_value(args[4].clone()));
        let tint = vm_try!(Color32::from_value(args[5].clone()));

        this.draw_text(&mut painter, &text, size as f32, position, tint);

        VmResult::Ok(())
    }

    fn draw_text_layout_wrapper(stack: &mut dyn Memory, addr: InstAddress, args: usize, _: Output) -> VmResult<()> {
        let args = vm_try!(stack.slice_at(addr, args));

        let mut this = vm_try!(args[0].borrow_mut::<Self>());
        let mut painter = vm_try!(args[1].borrow_mut::<Painter>());
        let text: String = vm_try!(String::from_value(args[2].clone()));
        let size = vm_try!(args[3].as_float());
        let position = vm_try!(Vector2::from_value(args[4].clone()));
        let pivot = vm_try!(Vector2::from_value(args[5].clone()));
        let rotation = vm_try!(args[6].as_float());
        let (width, height) = vm_try!(<(f32, f32)>::from_value(args[7].clone()));
        let h_align = vm_try!(HAlign::from_value(args[8].clone()));
        let v_align = vm_try!(VAlign::from_value(args[9].clone()));
        let wrap = vm_try!(TextWrap::from_value(args[10].clone()));
        let tint = vm_try!(Color32::from_value(args[11].clone()));

        let h_align = match h_align {
            HAlign::Left => HorizontalAlign::Left,
            HAlign::Middle => HorizontalAlign::Center,
            HAlign::Right => HorizontalAlign::Right
        };

        let v_align = match v_align {
            VAlign::Top => VerticalAlign::Top,
            VAlign::Middle => VerticalAlign::Middle,
            VAlign::Bottom => VerticalAlign::Bottom
        };

        let wrap = match wrap {
            TextWrap::Letter => WrapStyle::Letter,
            TextWrap::Word => WrapStyle::Word
        };

        this.draw_text_layout(&mut painter,
            &text,
            size as f32,
            position,
            pivot,
            rotation.to_radians() as f32,
            width,
            height,
            h_align,
            v_align,
            wrap,
            tint);

        VmResult::Ok(())
    }

    pub fn register_script(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<Self>()?;
        module.function_meta(Self::load)?;

        module.raw_function("draw_text", Self::draw_text_wrapper)
            .build_associated::<Self>()?
            .args(6)
            .argument_types::<(Self, Painter, String, f32, Vector2, Color32)>()?;

        module.raw_function("draw_text_layout", Self::draw_text_layout_wrapper)
            .build_associated::<Self>()?
            .args(12)
            .argument_types::<(Self, Painter, String, f32, Vector2, Vector2, f32, (f32, f32), HAlign, VAlign, TextWrap, Color32)>()?;

        Ok(())
    }
}

#[derive(Any)]
struct Painter {
    #[allow(unused)]
    painter: UiPainter,
}

impl Painter {
    pub fn new(max_quads: usize) -> Painter {
        Painter {
            painter: UiPainter::new(max_quads)
        }
    }

    pub fn begin(&mut self, window_size: (u32, u32)) {
        self.painter.begin(window_size);
    }

    pub fn end(&mut self) {
        self.painter.end();
    }

    pub fn draw_sprite(&mut self, texture: &Texture, position: Vector2, size: Option<Vector2>, pivot: Vector2, rotation: f32, tex_rect: Option<Rectangle>, tint: Color32) {
        let size = if let Some(v) = size {
            v
        }
        else {
            Vector2::new(texture.texture.width() as f32, texture.texture.height() as f32)
        };

        let tex_rect = if let Some(v) = tex_rect {
            v
        }
        else {
            Rectangle::new(0, 0, texture.texture.width(), texture.texture.height())
        };

        self.painter.draw_sprite(&texture.texture, position, size, pivot * size, rotation, tex_rect, tint);
    }

    pub fn draw_nineslice(&mut self, texture: &Texture,
        position: Vector2,
        size: Vector2,
        pivot: Vector2,
        rotation: f32,
        tex_rect: Option<Rectangle>,
        borders: (i32, i32, i32, i32),
        color: Color32)
    {
        self.painter.draw_nineslice(&texture.texture, position, size, pivot * size, rotation, tex_rect, borders, color);
    }

    fn draw_sprite_wrapper(stack: &mut dyn Memory, addr: InstAddress, args: usize, _: Output) -> VmResult<()> {
        let args = vm_try!(stack.slice_at(addr, args));

        let mut this = vm_try!(args[0].borrow_mut::<Self>());
        let texture = vm_try!(args[1].borrow_ref::<Texture>());
        let position = vm_try!(Vector2::from_value(args[2].clone()));
        let size = vm_try!(Option::<Vector2>::from_value(args[3].clone()));
        let pivot = vm_try!(Vector2::from_value(args[4].clone()));
        let rotation = vm_try!(args[5].as_float());
        let tex_rect = vm_try!(Option::<Rectangle>::from_value(args[6].clone()));
        let tint = vm_try!(Color32::from_value(args[7].clone()));

        this.draw_sprite(&texture, position, size, pivot, rotation.to_radians() as f32, tex_rect, tint);

        VmResult::Ok(())
    }

    fn draw_nineslice_wrapper(stack: &mut dyn Memory, addr: InstAddress, args: usize, _: Output) -> VmResult<()> {
        let args = vm_try!(stack.slice_at(addr, args));

        let mut this = vm_try!(args[0].borrow_mut::<Self>());
        let texture = vm_try!(args[1].borrow_ref::<Texture>());
        let position = vm_try!(Vector2::from_value(args[2].clone()));
        let size = vm_try!(Vector2::from_value(args[3].clone()));
        let pivot = vm_try!(Vector2::from_value(args[4].clone()));
        let rotation = vm_try!(args[5].as_float());
        let tex_rect = vm_try!(Option::<Rectangle>::from_value(args[6].clone()));
        let borders = vm_try!(<(i32, i32, i32, i32)>::from_value(args[7].clone()));
        let tint = vm_try!(Color32::from_value(args[8].clone()));

        this.draw_nineslice(&texture, position, size, pivot, rotation.to_radians() as f32, tex_rect, borders, tint);

        VmResult::Ok(())
    }

    pub fn register_script(module: &mut Module) -> Result<(), ContextError> {
        module.ty::<Self>()?;

        module.raw_function("draw_sprite", Self::draw_sprite_wrapper)
            .build_associated::<Self>()?
            .args(8)
            .argument_types::<(Self, Texture, Vector2, Option<Vector2>, Vector2, f32, Option<Rectangle>, Color32)>()?;

        module.raw_function("draw_nineslice", Self::draw_nineslice_wrapper)
            .build_associated::<Self>()?
            .args(9)
            .argument_types::<(Self, Texture, Vector2, Vector2, Vector2, f32, Option<Rectangle>, (i32, i32, i32, i32), Color32)>()?;

        Ok(())
    }
}

fn module() -> Result<Module, ContextError> {
    let mut m = Module::new();
    
    Font::register_script(&mut m)?;
    Texture::register_script(&mut m)?;
    Painter::register_script(&mut m)?;
    Vector2::register_script(&mut m)?;
    Rectangle::register_script(&mut m)?;
    Color32::register_script(&mut m)?;

    m.ty::<HAlign>()?;
    m.ty::<VAlign>()?;
    m.ty::<TextWrap>()?;

    Ok(m)
}

pub struct UiScript {
    _vm: Vm,
    painter: Painter,
    instance: Value,
    update_fn: Function,
    paint_fn: Function,
}

impl UiScript {
    pub fn new(script_path: &str, type_name: &str) -> UiScript {
        let module = module().unwrap();

        let mut context = Context::with_default_modules().unwrap();
        context.install(module).unwrap();

        let runtime = Arc::new(context.runtime().unwrap());

        let mut sources = Sources::new();
        sources.insert(Source::from_path(script_path).unwrap()).unwrap();

        let mut diagnostics = Diagnostics::new();

        let result = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources).unwrap();
        }

        let unit = result.unwrap();
        let mut vm = Vm::new(runtime, Arc::new(unit));

        let update_fn = vm.lookup_function([type_name, "update"]).unwrap();
        let paint_fn = vm.lookup_function([type_name, "paint"]).unwrap();

        // construct new instance of script type
        let instance = vm.call([type_name, "new"], ()).unwrap();
        
        UiScript {
            _vm: vm,
            painter: Painter::new(1024),
            instance,
            update_fn,
            paint_fn,
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.update_fn.call::<()>((&self.instance, delta,)).unwrap();
    }

    pub fn paint(&mut self, window_size: (u32, u32)) {
        self.painter.begin(window_size);
        self.paint_fn.call::<()>((&self.instance, window_size.0 as i32, window_size.1 as i32, &mut self.painter,)).unwrap();
        self.painter.end();
    }
}