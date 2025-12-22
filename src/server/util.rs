use std::{
    fs,
    hash::{Hash, Hasher},
};

use ahash::AHasher;

#[inline(always)]
pub fn lerp_f64(pos: f64, x: f64, y: f64) -> f64 {
    x + (pos * (y - x))
}

#[inline(always)]
pub fn lerp_f32(pos: f32, x: f32, y: f32) -> f32 {
    x + (pos * (y - x))
}

#[inline(always)]
pub fn inverse_lerp_f64(pos: f64, x: f64, y: f64) -> f64 {
    (pos - x) / (y - x)
}

#[inline(always)]
pub fn lerp2_f64(pos1: f64, pos2: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    lerp_f64(pos2, lerp_f64(pos1, x1, y1), lerp_f64(pos1, x2, y2))
}

#[inline(always)]
pub fn lerp3_f64(
    pos1: f64,
    pos2: f64,
    pos3: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    x3: f64,
    y3: f64,
    x4: f64,
    y4: f64,
) -> f64 {
    lerp_f64(
        pos3,
        lerp2_f64(pos1, pos2, x1, y1, x2, y2),
        lerp2_f64(pos1, pos2, x3, y3, x4, y4),
    )
}

#[inline(always)]
pub fn smoothstep(value: f64) -> f64 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
pub fn clamped_map_f64(pos1: f64, x: f64, y: f64, val1: f64, val2: f64) -> f64 {
    let inv_lerp = inverse_lerp_f64(pos1, x, y);
    if inv_lerp < 0.0 {
        val1
    } else if inv_lerp > 1.0 {
        val2
    } else {
        lerp_f64(inv_lerp, val1, val2)
    }
}

pub fn get_dir_files(
    dir: fs::ReadDir,
    files: &mut Vec<(String, fs::File)>,
    rel_path: &str,
) -> std::io::Result<()> {
    for entry in dir {
        let rel_path = if !rel_path.is_empty() {
            rel_path.to_string() + "/"
        } else {
            rel_path.to_string()
        };
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let new_rel_path = format!("{}{}", rel_path, entry.file_name().to_str().unwrap());
            let entry_dir = entry
                .path()
                .read_dir()
                .expect("Error reading density function sub-directory");
            get_dir_files(entry_dir, files, &new_rel_path).unwrap();
        } else {
            files.push((
                format!(
                    "{}{}",
                    rel_path,
                    entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .split_once(".")
                        .unwrap()
                        .0
                ),
                fs::File::open(entry.path()).expect("Error reading density function file"),
            ));
        }
    }
    Ok(())
}

pub fn get_noise_key(dimension: &str, noise_path: &str) -> u64 {
    let mut hasher = AHasher::default();
    dimension.hash(&mut hasher);
    noise_path.hash(&mut hasher);
    hasher.finish()
}
