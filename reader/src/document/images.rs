use super::{local_path, read_bounded};
use std::path::Path;

const MAX_IMAGE: u64 = 8 * 1024 * 1024;
const MAX_ASSETS: usize = 24 * 1024 * 1024;
#[derive(Default)]
pub(crate) struct ImageBudget {
    bytes: usize,
    pixels: u64,
    count: usize,
}

pub(crate) struct LocalImage {
    pub bytes: Vec<u8>,
    pub(super) mime: &'static str,
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(crate) fn read_local_image(
    root: &Path,
    dest: &str,
    budget: &mut ImageBudget,
) -> Result<LocalImage, String> {
    let decoded = percent_encoding::percent_decode_str(dest)
        .decode_utf8()
        .map_err(|_| "画像パスが不正です")?;
    let dest = decoded.as_ref();
    if dest.contains(':') || dest.starts_with(['/', '\\']) {
        return Err("外部画像・絶対パスは読み込みません".into());
    }
    if budget.count >= 64 {
        return Err("画像の個数上限です".into());
    }
    budget.count += 1;
    let file = local_path(&root.join(dest)).map_err(|_| "画像が見つかりません")?;
    if !file.starts_with(root) {
        return Err("参照フォルダー外の画像です".into());
    }
    let mime = match file
        .extension()
        .and_then(|s| s.to_str())
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => return Err("未対応の画像形式です".into()),
    };
    let bytes = read_bounded(&file, MAX_IMAGE)?;
    if budget.bytes + bytes.len() > MAX_ASSETS {
        return Err("画像の合計サイズ上限です".into());
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|_| "画像を確認できません")?;
    let (w, h) = reader.into_dimensions().map_err(|_| "画像が壊れています")?;
    if u64::from(w) * u64::from(h) > 20_000_000 {
        return Err("画像の画素数上限です".into());
    }
    let area = u64::from(w) * u64::from(h);
    if budget.pixels + area > 40_000_000 {
        return Err("画像の合計画素数上限です".into());
    }
    budget.pixels += area;
    budget.bytes += bytes.len();
    Ok(LocalImage {
        bytes,
        mime,
        width: w,
        height: h,
    })
}
