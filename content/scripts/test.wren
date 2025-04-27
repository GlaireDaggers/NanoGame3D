import "engine" for Painter, Texture, Font, HorizontalAlign, VerticalAlign

class Test {
    construct new() {
        _painter = Painter.new(1024)
        _tex = Texture.load("content/textures/effects/glow.basis")
        _font = Font.load("content/fonts/Roboto-Regular.ttf")
        _rot = 0
    }

    update(dt) {
        _rot = _rot + (dt * 45)
    }

    paint(screen_width, screen_height) {
        _painter.begin(screen_width, screen_height)

        _painter.draw_sprite(
            _tex,
            400, 100,                           // position
            _tex.width, _tex.height,            // size
            0.5, 0.5,                           // pivot
            _rot,                               // rotation
            0, 0, _tex.width, _tex.height,      // texture rect
            1.0, 1.0, 1.0, 1.0)                 // color

        _font.draw_text(_painter,
            "Hello, NanoGame3D!",
            18,                     // size
            16, 16,                 // position
            1.0, 1.0, 1.0, 1.0)     // color

        _font.draw_text(_painter,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. In metus ante, auctor eget nulla eget, blandit tempus elit. Nam sollicitudin ligula at ipsum viverra, ac congue nunc dignissim. Vestibulum iaculis quam finibus, pulvinar tortor id, dapibus risus. Etiam ut congue orci. Cras nisl ipsum, tristique vel ante et, dapibus laoreet erat. Praesent iaculis nunc id elit porta molestie sed a nisl. Morbi eu felis at nibh accumsan tincidunt. Vivamus in orci luctus, pellentesque leo nec, lobortis metus.",
            18,                     // size
            16, 40,                 // position
            256, 256,               // width/height
            HorizontalAlign.LEFT,
            VerticalAlign.TOP,
            1.0, 0.1, 0.1, 1.0)     // color

        _painter.end()
    }
}