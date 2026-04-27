use std::fs;
use std::path::Path;

fn main() {
    // 移行元と移行先のディレクトリを定義
    let source_dir = r"D:\download";
    let destination_dir = r"C:\Users\array\OneDrive\画像";

    // 画像の拡張子リスト
    let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic"];

    // 宛先ディレクトリが存在しない場合は作成
    if let Err(e) = fs::create_dir_all(destination_dir) {
        eprintln!("エラー: 宛先ディレクトリの作成に失敗しました: {}", e);
        return;
    }
    println!(
        "対象ディレクトリ: '{}' が存在しない場合、作成しました。",
        destination_dir
    );
    println!(
        "'{}' から '{}' へ画像ファイルを移動します...",
        source_dir, destination_dir
    );

    // 移行元ディレクトリの読み込み
    let entries = match fs::read_dir(source_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("エラー: ソースディレクトリの読み取りに失敗しました: {}", e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // ファイルであるか、また拡張子をチェック
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();

                if image_extensions.contains(&ext_lower.as_str()) {
                    let filename = entry.file_name();
                    let dest_path = Path::new(destination_dir).join(&filename);
                    let filename_str = filename.to_string_lossy();

                    // DドライブからCドライブなど、別ドライブ間での rename はOSレベルで失敗するため、
                    // rename が失敗した場合は copy & remove にフォールバックする
                    match fs::rename(&path, &dest_path) {
                        Ok(_) => println!("移動しました: '{}'", filename_str),
                        Err(_) => {
                            // renameに失敗した場合（別ドライブ間など）
                            match fs::copy(&path, &dest_path) {
                                Ok(_) => {
                                    // コピー成功後に元のファイルを削除
                                    if let Err(e) = fs::remove_file(&path) {
                                        eprintln!(
                                            "警告: '{}' はコピーされましたが、元のファイルの削除に失敗しました: {}",
                                            filename_str, e
                                        );
                                    } else {
                                        println!("移動しました: '{}'", filename_str);
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "エラー: '{}' の移動中に予期せぬエラーが発生しました: {}",
                                        filename_str, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("画像ファイルの移動が完了しました。");
}
