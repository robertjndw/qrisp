//! Turns the dark modules of a symbol into one compound vector path.
//!
//! The output is a Typst array expression like `(M((1*u,2*u)),L((3*u,2*u)),Z)` in
//! module units. src/render.typ evals it with `M`, `L`, `C`, `Z` bound to the curve
//! constructors and `u` to the module size.
//!
//! We used to return raw f32s, but decoding them one by one in the Typst interpreter
//! took longer than everything else combined. `eval` parses the text natively.
//! One path rather than one shape per module also avoids anti-aliasing seams.

const OP_MOVE: &str = "M";
const OP_LINE: &str = "L";
const OP_CUBIC: &str = "C";
const OP_CLOSE: &str = "Z";

/// Control point distance for approximating a quarter circle with a cubic Bezier.
const KAPPA: f32 = 0.552_284_8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Square,
    Rounded,
    Dots,
}

// Directions in y-down coordinates, ordered so that `d + 1` is a right turn.
const DIRS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const RIGHT: u8 = 0;
const DOWN: u8 = 1;
const LEFT: u8 = 2;
const UP: u8 = 3;

/// A polygon corner: the vertex plus the incoming and outgoing directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Corner {
    x: i32,
    y: i32,
    din: u8,
    dout: u8,
}

/// Traces the boundary of the union of dark cells into closed loops of corners.
///
/// Every boundary edge is oriented so the dark side is on its right. Outer boundaries
/// therefore run clockwise and holes counter-clockwise (on screen), so the nonzero
/// fill rule gets holes right. Where two dark cells only touch diagonally, the
/// tracer always turns right, so each cell keeps its own corner and contours never
/// cross themselves.
fn trace(size: i32, dark: impl Fn(i32, i32) -> bool) -> Vec<Vec<Corner>> {
    let stride = size + 1;
    let vid = |x: i32, y: i32| (y * stride + x) as usize;
    let mut out = vec![0u8; (stride * stride) as usize];

    for y in 0..size {
        for x in 0..size {
            if !dark(x, y) {
                continue;
            }
            if !dark(x, y - 1) {
                out[vid(x, y)] |= 1 << RIGHT;
            }
            if !dark(x + 1, y) {
                out[vid(x + 1, y)] |= 1 << DOWN;
            }
            if !dark(x, y + 1) {
                out[vid(x + 1, y + 1)] |= 1 << LEFT;
            }
            if !dark(x - 1, y) {
                out[vid(x, y + 1)] |= 1 << UP;
            }
        }
    }

    // Picks the edge leaving `v` after arriving with direction `d`. The choice must not
    // depend on which edges were already visited, otherwise a loop that starts at a
    // diagonal vertex could wander into its neighbour instead of closing.
    let next_dir = |mask: u8, d: u8| -> u8 {
        if mask.count_ones() == 1 {
            mask.trailing_zeros() as u8
        } else {
            (d + 1) % 4
        }
    };

    let mut used = vec![0u8; out.len()];
    let mut loops = Vec::new();
    for sy in 0..stride {
        for sx in 0..stride {
            for sd in 0..4u8 {
                let start = vid(sx, sy);
                if out[start] & (1 << sd) == 0 || used[start] & (1 << sd) != 0 {
                    continue;
                }
                let mut corners = Vec::new();
                let (mut x, mut y, mut d) = (sx, sy, sd);
                loop {
                    used[vid(x, y)] |= 1 << d;
                    x += DIRS[d as usize].0;
                    y += DIRS[d as usize].1;
                    let nd = next_dir(out[vid(x, y)], d);
                    if nd != d {
                        corners.push(Corner { x, y, din: d, dout: nd });
                    }
                    if used[vid(x, y)] & (1 << nd) != 0 {
                        break;
                    }
                    d = nd;
                }
                loops.push(corners);
            }
        }
    }
    loops
}

struct Path(Vec<u8>);

impl Path {
    fn op(&mut self, op: &str, pts: &[f32]) {
        if self.0.len() > 1 {
            self.0.push(b',');
        }
        self.0.extend_from_slice(op.as_bytes());
        if pts.is_empty() {
            return;
        }
        self.0.push(b'(');
        for (i, xy) in pts.chunks(2).enumerate() {
            if i > 0 {
                self.0.push(b',');
            }
            self.0.push(b'(');
            push_num(&mut self.0, xy[0]);
            self.0.extend_from_slice(b"*u,");
            push_num(&mut self.0, xy[1]);
            self.0.extend_from_slice(b"*u)");
        }
        self.0.push(b')');
    }

    /// The finished array expression. A trailing comma keeps a single-element
    /// result an array rather than a parenthesised expression.
    fn into_code(mut self) -> Vec<u8> {
        if self.0.len() == 1 {
            return b"()".to_vec();
        }
        self.0.extend_from_slice(b",)");
        self.0
    }
}

/// Four decimals (1e-4 of a module) is plenty. Hand-rolled because the std float
/// formatter is slow under Typst's wasm interpreter and showed up as most of the
/// plugin's run time.
fn push_num(out: &mut Vec<u8>, v: f32) {
    let scaled = (v * 10_000.0).round() as i64;
    if scaled < 0 {
        out.push(b'-');
    }
    let abs = scaled.unsigned_abs();
    let (int, mut frac) = (abs / 10_000, abs % 10_000);
    push_uint(out, int);
    if frac != 0 {
        out.push(b'.');
        let mut width = 4;
        while frac % 10 == 0 {
            frac /= 10;
            width -= 1;
        }
        let digits = frac.ilog10() as usize + 1;
        out.extend(std::iter::repeat(b'0').take(width - digits));
        push_uint(out, frac);
    }
}

fn push_uint(out: &mut Vec<u8>, mut v: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    out.extend_from_slice(&buf[i..]);
}

fn emit_loop(path: &mut Path, corners: &[Corner], radius: f32) {
    let n = corners.len();
    if n == 0 {
        return;
    }
    let ends = |c: &Corner| {
        let (ix, iy) = DIRS[c.din as usize];
        let (ox, oy) = DIRS[c.dout as usize];
        let (px, py) = (c.x as f32, c.y as f32);
        let a = (px - ix as f32 * radius, py - iy as f32 * radius);
        let b = (px + ox as f32 * radius, py + oy as f32 * radius);
        let c1 = (a.0 + ix as f32 * radius * KAPPA, a.1 + iy as f32 * radius * KAPPA);
        let c2 = (b.0 - ox as f32 * radius * KAPPA, b.1 - oy as f32 * radius * KAPPA);
        (a, c1, c2, b)
    };

    let (_, _, _, start) = ends(&corners[0]);
    path.op(OP_MOVE, &[start.0, start.1]);
    for i in 1..=n {
        let c = &corners[i % n];
        if radius > 0.0 {
            let (a, c1, c2, b) = ends(c);
            path.op(OP_LINE, &[a.0, a.1]);
            path.op(OP_CUBIC, &[c1.0, c1.1, c2.0, c2.1, b.0, b.1]);
        } else if i < n {
            path.op(OP_LINE, &[c.x as f32, c.y as f32]);
        }
    }
    path.op(OP_CLOSE, &[]);
}

fn emit_circle(path: &mut Path, cx: f32, cy: f32, r: f32) {
    let k = r * KAPPA;
    path.op(OP_MOVE, &[cx + r, cy]);
    path.op(OP_CUBIC, &[cx + r, cy + k, cx + k, cy + r, cx, cy + r]);
    path.op(OP_CUBIC, &[cx - k, cy + r, cx - r, cy + k, cx - r, cy]);
    path.op(OP_CUBIC, &[cx - r, cy - k, cx - k, cy - r, cx, cy - r]);
    path.op(OP_CUBIC, &[cx + k, cy - r, cx + r, cy - k, cx + r, cy]);
    path.op(OP_CLOSE, &[]);
}

/// Builds the path for all cells where `dark` is true.
///
/// `radius` is the rounding strength in 0..=1. For `Rounded` it scales the corner
/// radius up to half a module (so a lone module becomes a circle); convex corners are
/// rounded off and concave ones get a matching fillet. For `Dots` it scales the dot
/// diameter from 70% to 100% of a module.
pub fn build(size: i32, style: Style, radius: f32, dark: impl Fn(i32, i32) -> bool) -> Vec<u8> {
    let radius = radius.clamp(0.0, 1.0);
    let mut path = Path(vec![b'(']);
    match style {
        Style::Square | Style::Rounded => {
            let r = if style == Style::Rounded { radius * 0.5 } else { 0.0 };
            for corners in trace(size, dark) {
                emit_loop(&mut path, &corners, r);
            }
        }
        Style::Dots => {
            let r = 0.35 + 0.15 * radius;
            for y in 0..size {
                for x in 0..size {
                    if dark(x, y) {
                        emit_circle(&mut path, x as f32 + 0.5, y as f32 + 0.5, r);
                    }
                }
            }
        }
    }
    path.into_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nonzero winding test of a point against the square-cornered polygons.
    fn inside(loops: &[Vec<Corner>], px: f32, py: f32) -> bool {
        let mut winding = 0;
        for corners in loops {
            let n = corners.len();
            for i in 0..n {
                let (a, b) = (corners[i], corners[(i + 1) % n]);
                let (ax, ay, bx, by) = (a.x as f32, a.y as f32, b.x as f32, b.y as f32);
                if ay <= py && by > py && (bx - ax) * (py - ay) - (px - ax) * (by - ay) > 0.0 {
                    winding += 1;
                } else if ay > py && by <= py && (bx - ax) * (py - ay) - (px - ax) * (by - ay) < 0.0 {
                    winding -= 1;
                }
            }
        }
        winding != 0
    }

    fn check_grid(size: i32, cells: &[bool]) {
        let dark = |x: i32, y: i32| x >= 0 && y >= 0 && x < size && y < size && cells[(y * size + x) as usize];
        let loops = trace(size, dark);
        for y in 0..size {
            for x in 0..size {
                assert_eq!(
                    inside(&loops, x as f32 + 0.5, y as f32 + 0.5),
                    dark(x, y),
                    "cell ({x}, {y}) of {cells:?}"
                );
            }
        }
        for corners in &loops {
            for c in corners {
                assert_ne!(c.din, c.dout);
                assert_ne!((c.din + 2) % 4, c.dout, "contour reverses on itself");
            }
        }
    }

    #[test]
    fn numbers_format_as_short_decimals() {
        let cases = [(0.0, "0"), (3.0, "3"), (1.5, "1.5"), (0.27614, "0.2761"), (-0.05, "-0.05"), (12.0001, "12.0001"), (-0.00001, "0")];
        for (v, want) in cases {
            let mut s = Vec::new();
            push_num(&mut s, v);
            assert_eq!(String::from_utf8(s).unwrap(), want, "{v}");
        }
    }

    #[test]
    fn single_cell_is_one_square() {
        let loops = trace(1, |x, y| x == 0 && y == 0);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].len(), 4);
    }

    #[test]
    fn diagonal_cells_stay_separate() {
        let loops = trace(2, |x, y| (x == 0 && y == 0) || (x == 1 && y == 1));
        assert_eq!(loops.len(), 2);
        assert!(loops.iter().all(|l| l.len() == 4));
    }

    #[test]
    fn ring_has_hole() {
        let cells: Vec<bool> = (0..9).map(|i| i != 4).collect();
        check_grid(3, &cells);
        assert_eq!(trace(3, |x, y| x >= 0 && y >= 0 && x < 3 && y < 3 && !(x == 1 && y == 1)).len(), 2);
    }

    #[test]
    fn exhaustive_4x4() {
        for bits in 0u32..(1 << 16) {
            let cells: Vec<bool> = (0..16).map(|i| bits & (1 << i) != 0).collect();
            check_grid(4, &cells);
        }
    }
}
