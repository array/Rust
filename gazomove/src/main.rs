use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// 設定を一箇所にまとめる
struct Config {
    source: &'static str,
    destination: &'static str,
    extensions: &'static [&'static str],
}

fn main() -> Result<()> {
    let config = Config {
        source: r"D:\download",
        destination: r"C:\Users\array\OneDrive\画像",
        extensions: &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic"],
    };

    // 1. 準備
    fs::create_dir_all(config.destination)
        .with_context(|| format!("ディレクトリ作成失敗: {}", config.destination))?;

    println!("'{}' から画像を移動中...", config.source);

    // 2. 実行（メインロジックを分離）
    let moved_count = run_move(&config)?;

    println!(
        "完了しました。計 {} 個のファイルを移動しました。",
        moved_count
    );
    Ok(())
}

/// 実際の移動ロジック
fn run_move(config: &Config) -> Result<u32> {
    let mut count = 0;
    let dest_root = Path::new(config.destination);

    // WalkDirなら再帰処理がたったこれだけ！
    for entry in WalkDir::new(config.source)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if is_image_file(path, config.extensions) {
            let filename = path.file_name().unwrap();
            let dest_path = dest_root.join(filename);

            move_file(path, &dest_path)?;
            println!("移動: {:?}", filename);
            count += 1;
        }
    }
    Ok(count)
}

/// 画像かどうか判定
fn is_image_file(path: &Path, extensions: &[&str]) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| extensions.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
}

/// ファイル移動（ドライブ跨ぎ対応）
fn move_file(from: &Path, to: &Path) -> Result<()> {
    if fs::rename(from, to).is_err() {
        fs::copy(from, to)?;
        fs::remove_file(from)?;
    }
    Ok(())
}
