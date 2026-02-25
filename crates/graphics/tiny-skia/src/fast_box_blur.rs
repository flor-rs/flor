use tiny_skia::PremultipliedColorU8;

/// An O(1) Fast Box Blur approximation of Gaussian blur for PremultipliedColorU8
/// Works directly on the flat pixel slice of a tiny-skia Pixmap.
pub fn fast_box_blur(
    pixels: &mut [PremultipliedColorU8],
    width: usize,
    height: usize,
    radius: u32,
) {
    if radius == 0 || width == 0 || height == 0 {
        return;
    }

    let r = radius as usize;
    // Approximating Gaussian blur with 3 passes of Box Blur
    box_blur(pixels, width, height, r);
    box_blur(pixels, width, height, r);
    box_blur(pixels, width, height, r);
}

fn box_blur(pixels: &mut [PremultipliedColorU8], w: usize, h: usize, r: usize) {
    let mut buffer = pixels.to_vec();
    box_blur_h(&mut buffer, pixels, w, h, r);
    box_blur_v(pixels, &mut buffer, w, h, r);
    pixels.copy_from_slice(&buffer);
}

fn box_blur_h(
    scl: &[PremultipliedColorU8],
    tcl: &mut [PremultipliedColorU8],
    w: usize,
    h: usize,
    r: usize,
) {
    let arr = r as u32 * 2 + 1;
    for i in 0..h {
        let mut ti = i * w;
        let mut li = ti;
        let mut ri = ti + r + 1;

        // Edge cases initialization
        let mut fr = 0;
        let mut fg = 0;
        let mut fb = 0;
        let mut fa = 0;
        let fv = scl[ti];
        let lv = scl[ti + w - 1];

        for _ in 0..r {
            fr += fv.red() as u32;
            fg += fv.green() as u32;
            fb += fv.blue() as u32;
            fa += fv.alpha() as u32;
        }
        for j in 0..=r {
            if ti + j < scl.len() {
                let c = scl[ti + j];
                fr += c.red() as u32;
                fg += c.green() as u32;
                fb += c.blue() as u32;
                fa += c.alpha() as u32;
            }
        }
        for j in 0..w {
            let pr = fr / arr;
            let pg = fg / arr;
            let pb = fb / arr;
            let pa = fa / arr;
            tcl[ti] = PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, pa as u8)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
            ti += 1;

            let val_right = if j + r + 1 < w && ri < scl.len() {
                let c = scl[ri];
                ri += 1;
                c
            } else {
                lv
            };
            let val_left = if j >= r && li < scl.len() {
                let c = scl[li];
                li += 1;
                c
            } else {
                fv
            };

            fr += val_right.red() as u32;
            fg += val_right.green() as u32;
            fb += val_right.blue() as u32;
            fa += val_right.alpha() as u32;

            fr = fr.saturating_sub(val_left.red() as u32);
            fg = fg.saturating_sub(val_left.green() as u32);
            fb = fb.saturating_sub(val_left.blue() as u32);
            fa = fa.saturating_sub(val_left.alpha() as u32);
        }
    }
}

fn box_blur_v(
    scl: &[PremultipliedColorU8],
    tcl: &mut [PremultipliedColorU8],
    w: usize,
    h: usize,
    r: usize,
) {
    let arr = r as u32 * 2 + 1;
    for i in 0..w {
        let mut ti = i;
        let mut li = ti;
        let mut ri = ti + (r + 1) * w;

        let mut fr = 0;
        let mut fg = 0;
        let mut fb = 0;
        let mut fa = 0;
        let fv = scl[ti];
        let lv = scl[ti + w * (h - 1)];
        for _ in 0..r {
            fr += fv.red() as u32;
            fg += fv.green() as u32;
            fb += fv.blue() as u32;
            fa += fv.alpha() as u32;
        }
        for j in 0..=r {
            if ti + j * w < scl.len() {
                let c = scl[ti + j * w];
                fr += c.red() as u32;
                fg += c.green() as u32;
                fb += c.blue() as u32;
                fa += c.alpha() as u32;
            }
        }
        for j in 0..h {
            let pr = fr / arr;
            let pg = fg / arr;
            let pb = fb / arr;
            let pa = fa / arr;
            tcl[ti] = PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, pa as u8)
                .unwrap_or(PremultipliedColorU8::TRANSPARENT);
            ti += w;

            let val_below = if j + r + 1 < h && ri < scl.len() {
                let c = scl[ri];
                ri += w;
                c
            } else {
                lv
            };
            let val_above = if j >= r && li < scl.len() {
                let c = scl[li];
                li += w;
                c
            } else {
                fv
            };

            fr += val_below.red() as u32;
            fg += val_below.green() as u32;
            fb += val_below.blue() as u32;
            fa += val_below.alpha() as u32;

            fr = fr.saturating_sub(val_above.red() as u32);
            fg = fg.saturating_sub(val_above.green() as u32);
            fb = fb.saturating_sub(val_above.blue() as u32);
            fa = fa.saturating_sub(val_above.alpha() as u32);
        }
    }
}
