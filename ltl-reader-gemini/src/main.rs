#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use serde::Deserialize;
use std::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinSet;

// Mastodonのステータス構造体
#[derive(Deserialize, Clone)]
struct Status {
    id: String,
    created_at: String,
    content: String, // HTMLが含まれるので注意
    url: String,
    account: Account,
}

#[derive(Deserialize, Clone)]
struct Account {
    display_name: String,
    acct: String,
}

struct MyApp {
    statuses: Vec<Status>,
    receiver: Receiver<Vec<Status>>,
    tx: Sender<Vec<Status>>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // --- 1. フォントの設定 ---
        let mut fonts = egui::FontDefinitions::default();
        // Windowsのフォントパス（ファイル名は環境に合わせて確認してください）
        let font_path = "C:\\Windows\\Fonts\\NotoSansJP-VF.ttf";

        let mut visuals = egui::Visuals::light();
        // 文字色をデフォルトより一段階濃くする設定
        visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(30, 30, 30);
        cc.egui_ctx.set_visuals(visuals);

        match std::fs::read(font_path) {
            Ok(font_data) => {
                fonts.font_data.insert(
                    "noto_sans".to_owned(),
                    egui::FontData::from_owned(font_data),
                );

                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "noto_sans".to_owned());

                cc.egui_ctx.set_fonts(fonts);
                cc.egui_ctx.set_pixels_per_point(1.2);
            }
            Err(e) => {
                eprintln!("フォントの読み込みに失敗しました: {}", e);
            }
        }

        // --- 2. 通信用のチャンネル作成 ---
        let (tx, rx) = std::sync::mpsc::channel();

        // --- 3. 非同期フェッチタスクの開始 ---
        // 起動時に一度だけLTLを取得しにいく
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            fetch_ltl(tx_clone).await;
        });

        // --- 4. 自信（Self）を初期化して返す ---
        Self {
            statuses: Vec::new(),
            receiver: rx,
            tx,
        }
    }
} // ← ここが impl MyApp の閉じ括弧

async fn fetch_ltl(tx: Sender<Vec<Status>>) {
    let client = reqwest::Client::new();
    // Vivaldi.socialのLTLエンドポイント
    let url = "https://vivaldi.social/api/v1/timelines/public?local=true";

    if let Ok(response) = client.get(url).send().await {
        if let Ok(data) = response.json::<Vec<Status>>().await {
            let _ = tx.send(data);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // データの受信チェック
        if let Ok(new_statuses) = self.receiver.try_recv() {
            self.statuses = new_statuses;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Vivaldi.social LTL (Read Only)");

            // ★上部のボタンに統合し、クリック時の動作を記述
            if ui.button("🔄 最新の情報に更新").clicked() {
                // 視覚的に更新中だとわかるように一旦クリアするのもアリです
                // self.statuses.clear();

                let tx_clone = self.tx.clone();
                tokio::spawn(async move {
                    fetch_ltl(tx_clone).await;
                });
            }

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let re = regex::Regex::new(r"<[^>]*>").unwrap();
                for status in &self.statuses {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("👤 {}", status.account.display_name));
                            let plain_text = re.replace_all(&status.content, "");
                            ui.label(plain_text.to_string());

                            if ui.button("🔗 ブラウザで開く").clicked() {
                                let _ = webbrowser::open(&status.url);
                            }
                        });
                    });
                    ui.add_space(8.0);
                }
                // ★スクロールエリア内のボタンは削除してOKです
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    // 1. 画像ファイルを読み込む（バイナリに埋め込むのが楽です）
    let icon_bytes = include_bytes!("icon.png"); // srcと同じフォルダに置いた場合
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .to_rgba8();
    let (width, height) = image.dimensions();

    // 2. egui用のアイコンデータを作成
    let icon_data = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    // 3. オプションにセット
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(icon_data) // ここでアイコンを指定！
            .with_inner_size([400.0, 800.0]), // ついでに初期サイズも指定可能
        ..Default::default()
    };

    eframe::run_native(
        "Vivaldi LTL Monitor",
        native_options,
        Box::new(|cc| Box::new(MyApp::new(cc))),
    )
}
