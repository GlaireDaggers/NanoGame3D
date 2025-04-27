foreign class Painter {
    construct new(max_quads) {}
    foreign begin(width, height)
    foreign end()
    foreign draw_sprite(tex, px, py, sx, sy, ax, ay, rot, tx, ty, tw, th, cr, cg, cb, ca)
}

foreign class Texture {
    construct load(path) {}
    foreign width
    foreign height
}

foreign class Font {
    construct load(path) {}
    foreign draw_text(painter, text, size, px, py, cr, cg, cb, ca)
    foreign draw_text(painter, text, size, px, py, w, h, h_align, v_align, cr, cg, cb, ca)
}

class HorizontalAlign {
    static LEFT {
        return 0
    }

    static MIDDLE {
        return 1
    }
    
    static RIGHT {
        return 2
    }
}

class VerticalAlign {
    static TOP {
        return 0
    }

    static MIDDLE {
        return 1
    }
    
    static BOTTOM {
        return 2
    }
}