/// Minimal PNG encoder — no external dependencies.
///
/// Produces valid RGB PNGs using deflate "stored" blocks (uncompressed).
/// Output is bit-for-bit identical to what a compliant decoder expects.

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pixels: Vec<u8>, // row-major RGB
}

impl Canvas {
    pub fn new(width: u32, height: u32, fill: [u8; 3]) -> Self {
        let pixels = fill.iter().copied().cycle().take((width * height * 3) as usize).collect();
        Canvas { width, height, pixels }
    }

    pub fn set(&mut self, x: u32, y: u32, color: [u8; 3]) {
        let i = ((y * self.width + x) * 3) as usize;
        self.pixels[i..i + 3].copy_from_slice(&color);
    }

    /// Nearest-neighbour scale — pixel art friendly.
    pub fn scale(&self, factor: u32) -> Canvas {
        let w = self.width * factor;
        let h = self.height * factor;
        let mut out = Canvas::new(w, h, [0, 0, 0]);
        for sy in 0..self.height {
            for sx in 0..self.width {
                let i = ((sy * self.width + sx) * 3) as usize;
                let color = [self.pixels[i], self.pixels[i + 1], self.pixels[i + 2]];
                for dy in 0..factor {
                    for dx in 0..factor {
                        out.set(sx * factor + dx, sy * factor + dy, color);
                    }
                }
            }
        }
        out
    }

    pub fn encode_png(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]); // PNG signature

        write_chunk(&mut out, b"IHDR", &ihdr(self.width, self.height));

        let raw = self.filter_none_scanlines();
        write_chunk(&mut out, b"IDAT", &zlib_stored(&raw));

        write_chunk(&mut out, b"IEND", &[]);
        out
    }

    fn filter_none_scanlines(&self) -> Vec<u8> {
        let row_bytes = self.width as usize * 3;
        let mut raw = Vec::with_capacity((1 + row_bytes) * self.height as usize);
        for row in 0..self.height as usize {
            raw.push(0x00); // filter type: None
            raw.extend_from_slice(&self.pixels[row * row_bytes..(row + 1) * row_bytes]);
        }
        raw
    }
}

// ── PNG chunks ────────────────────────────────────────────────────────────────

fn ihdr(width: u32, height: u32) -> Vec<u8> {
    let mut v = Vec::with_capacity(13);
    v.extend_from_slice(&width.to_be_bytes());
    v.extend_from_slice(&height.to_be_bytes());
    v.push(8); // bit depth
    v.push(2); // color type: RGB
    v.push(0); // compression: deflate
    v.push(0); // filter: adaptive
    v.push(0); // interlace: none
    v
}

fn write_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(kind, data);
    out.extend_from_slice(&crc.to_be_bytes());
}

// ── zlib stored-block framing ─────────────────────────────────────────────────

fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01]; // CMF=deflate, FLG=low-compression
    let mut chunks = data.chunks(65535).peekable();
    while let Some(chunk) = chunks.next() {
        let is_last = chunks.peek().is_none();
        out.push(u8::from(is_last)); // BFINAL | BTYPE=00
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes()); // NLEN = one's complement
        out.extend_from_slice(chunk);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

// ── CRC-32 (PNG uses the standard ISO 3309 polynomial) ───────────────────────

fn crc32(a: &[u8], b: &[u8]) -> u32 {
    let table = crc_table();
    let mut c = 0xffff_ffffu32;
    for &byte in a.iter().chain(b) {
        c = table[((c ^ u32::from(byte)) & 0xff) as usize] ^ (c >> 8);
    }
    c ^ 0xffff_ffff
}

fn crc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    for n in 0u32..256 {
        let mut c = n;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xedb8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        t[n as usize] = c;
    }
    t
}

// ── Adler-32 (zlib checksum) ──────────────────────────────────────────────────

fn adler32(data: &[u8]) -> u32 {
    let (mut s1, mut s2) = (1u32, 0u32);
    for &b in data {
        s1 = (s1 + u32::from(b)) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}
