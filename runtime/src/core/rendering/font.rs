use std::collections::HashMap;

/// glyph width/height in the source bitmap, before any on-screen scale
pub(crate) const GLYPH_W: u32 = 5;
pub(crate) const GLYPH_H: u32 = 7;

/// how many glyph cells per atlas row. each cell is padded by one pixel on the
/// right/bottom so bilinear sampling never bleeds into a neighboring glyph
const ATLAS_COLS: u32 = 8;
const CELL_W: u32 = GLYPH_W + 1;
const CELL_H: u32 = GLYPH_H + 1;

/// a 5x7 dot-matrix font. covers a-z, 0-9, and a handful of symbols, uppercase only
/// (a-z parses lowercase input as uppercase since that's plenty for debug hud text
/// and halves the table size). rows top to bottom, `#`/`.` per pixel left to right
const GLYPHS: &[(char, [&str; 7])] = &[
    (' ', [".....", ".....", ".....", ".....", ".....", ".....", "....."]),
    ('0', [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."]),
    ('1', ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."]),
    ('2', [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"]),
    ('3', [".###.", "#...#", "....#", "..##.", "....#", "#...#", ".###."]),
    ('4', ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."]),
    ('5', ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."]),
    ('6', ["..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###."]),
    ('7', ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."]),
    ('8', [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."]),
    ('9', [".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##.."]),
    ('A', ["..#..", ".#.#.", "#...#", "#...#", "#####", "#...#", "#...#"]),
    ('B', ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."]),
    ('C', [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."]),
    ('D', ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."]),
    ('E', ["#####", "#....", "#....", "####.", "#....", "#....", "#####"]),
    ('F', ["#####", "#....", "#....", "####.", "#....", "#....", "#...."]),
    ('G', [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".###."]),
    ('H', ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('I', [".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."]),
    ('J', ["....#", "....#", "....#", "....#", "#...#", "#...#", ".###."]),
    ('K', ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"]),
    ('L', ["#....", "#....", "#....", "#....", "#....", "#....", "#####"]),
    ('M', ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "#...#"]),
    ('N', ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"]),
    ('O', [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('P', ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."]),
    ('Q', [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"]),
    ('R', ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"]),
    ('S', [".###.", "#...#", "#....", ".###.", "....#", "#...#", ".###."]),
    ('T', ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."]),
    ('U', ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('V', ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."]),
    ('W', ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"]),
    ('X', ["#...#", ".#.#.", "..#..", "..#..", "..#..", ".#.#.", "#...#"]),
    ('Y', ["#...#", ".#.#.", "..#..", "..#..", "..#..", "..#..", "..#.."]),
    ('Z', ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"]),
    ('.', [".....", ".....", ".....", ".....", ".....", "..##.", "..##."]),
    (':', [".....", "..##.", "..##.", ".....", "..##.", "..##.", "....."]),
    ('-', [".....", ".....", ".....", "#####", ".....", ".....", "....."]),
    ('%', ["##..#", "##.#.", "...#.", "..#..", ".#...", ".#.##", "#..##"]),
    ('/', ["....#", "...#.", "..#..", "..#..", ".#...", ".#...", "#...."]),
];

/// uv rect (u_min, v_min, u_max, v_max) for one glyph within the atlas texture
pub(crate) type GlyphUv = [f32; 4];

/// a small procedurally-built bitmap font atlas: one r8 texture holding every glyph
/// this engine can draw, plus a lookup from character to its uv rect
pub(crate) struct FontAtlas {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub glyph_uv: HashMap<char, GlyphUv>,
}

impl FontAtlas {
    /// builds the atlas pixel buffer and glyph uv table. call once at startup
    pub fn build() -> Self {
        let rows = GLYPHS.len().div_ceil(ATLAS_COLS as usize) as u32;
        let width = ATLAS_COLS * CELL_W;
        let height = rows * CELL_H;
        let mut pixels = vec![0u8; (width * height) as usize];
        let mut glyph_uv = HashMap::new();

        for (i, (ch, rows_pattern)) in GLYPHS.iter().enumerate() {
            let cell_col = i as u32 % ATLAS_COLS;
            let cell_row = i as u32 / ATLAS_COLS;
            let origin_x = cell_col * CELL_W;
            let origin_y = cell_row * CELL_H;

            for (y, row) in rows_pattern.iter().enumerate() {
                for (x, pixel) in row.chars().enumerate() {
                    if pixel == '#' {
                        let px = origin_x + x as u32;
                        let py = origin_y + y as u32;
                        pixels[(py * width + px) as usize] = 255;
                    }
                }
            }

            glyph_uv.insert(
                *ch,
                [
                    origin_x as f32 / width as f32,
                    origin_y as f32 / height as f32,
                    (origin_x + GLYPH_W) as f32 / width as f32,
                    (origin_y + GLYPH_H) as f32 / height as f32,
                ],
            );
        }

        Self { pixels, width, height, glyph_uv }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_size_fits_all_glyphs() {
        let atlas = FontAtlas::build();
        let rows = GLYPHS.len().div_ceil(ATLAS_COLS as usize) as u32;
        assert_eq!(atlas.width, ATLAS_COLS * CELL_W);
        assert_eq!(atlas.height, rows * CELL_H);
    }

    #[test]
    fn every_glyph_has_a_uv_rect() {
        let atlas = FontAtlas::build();
        for (ch, _) in GLYPHS {
            assert!(atlas.glyph_uv.contains_key(ch), "missing uv for {ch:?}");
        }
    }

    #[test]
    fn space_glyph_is_blank() {
        let atlas = FontAtlas::build();
        let uv = atlas.glyph_uv[&' '];
        let x0 = (uv[0] * atlas.width as f32).round() as u32;
        let y0 = (uv[1] * atlas.height as f32).round() as u32;
        for y in 0..GLYPH_H {
            for x in 0..GLYPH_W {
                assert_eq!(atlas.pixels[((y0 + y) * atlas.width + x0 + x) as usize], 0);
            }
        }
    }

    #[test]
    fn letter_a_has_lit_pixels() {
        let atlas = FontAtlas::build();
        let uv = atlas.glyph_uv[&'A'];
        let x0 = (uv[0] * atlas.width as f32).round() as u32;
        let y0 = (uv[1] * atlas.height as f32).round() as u32;
        let lit: u32 = (0..GLYPH_H)
            .flat_map(|y| (0..GLYPH_W).map(move |x| (x, y)))
            .filter(|&(x, y)| atlas.pixels[((y0 + y) * atlas.width + x0 + x) as usize] > 0)
            .count() as u32;
        assert!(lit > 0);
    }
}
