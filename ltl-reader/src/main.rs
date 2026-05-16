#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize, Clone)]
struct Account {
    username: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Status {
    id: String,
    account: Account,
    content: String,
    created_at: String,
}

fn strip_html(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

fn to_jst(utc_str: &str) -> String {
    let jst = FixedOffset::east_opt(9 * 3600).unwrap();
    match utc_str.parse::<DateTime<Utc>>() {
        Ok(dt) => (dt.with_timezone(&jst))
            .format("%Y-%m-%d %H:%M:%S JST")
            .to_string(),
        Err(_) => utc_str.to_string(),
    }
}

struct LtlApp {
    statuses: Arc<Mutex<Vec<Status>>>,
    last_fetch: Instant,
}

impl LtlApp {
    fn new(cc: &eframe::CreationContext<'_>, statuses: Arc<Mutex<Vec<Status>>>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "meiryo".to_owned(),
            egui::FontData::from_static(include_bytes!("C:\\Windows\\Fonts\\meiryo.ttc")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "meiryo".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            statuses,
            last_fetch: Instant::now(),
        }
    }
}

impl eframe::App for LtlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Vivaldi Social LTL Reader");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🌐 Vivaldi Socialを開く").clicked() {
                        let _ = open::that("https://social.vivaldi.net");
                    }
                });
            });

            let elapsed = self.last_fetch.elapsed().as_secs();
            ui.label(
                egui::RichText::new(format!("最終更新: {}秒前", elapsed))
                    .size(12.0)
                    .color(egui::Color32::GRAY),
            );
            ui.separator();

            let statuses = self.statuses.lock().unwrap();

            if statuses.is_empty() {
                ui.label("取得中...");
                return;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for status in statuses.iter() {
                    ui.group(|ui| {
                        ui.label(
                            egui::RichText::new(format!("@{}", status.account.username))
                                .strong()
                                .color(egui::Color32::from_rgb(100, 150, 255)),
                        );
                        ui.label(strip_html(&status.content));
                        ui.label(
                            egui::RichText::new(to_jst(&status.created_at))
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }
}

async fn fetch_statuses(statuses: Arc<Mutex<Vec<Status>>>) {
    let url = "https://social.vivaldi.net/api/v1/timelines/public?local=true&limit=20";
    loop {
        if let Ok(resp) = reqwest::get(url).await {
            if let Ok(new_statuses) = resp.json::<Vec<Status>>().await {
                let mut lock = statuses.lock().unwrap();
                *lock = new_statuses;
            }
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let statuses: Arc<Mutex<Vec<Status>>> = Arc::new(Mutex::new(vec![]));
    let statuses_clone = Arc::clone(&statuses);

    tokio::spawn(async move {
        fetch_statuses(statuses_clone).await;
    });

    let icon_data = {
        let img = image::load_from_memory(include_bytes!("icon.png"))
            .expect("アイコン読み込み失敗")
            .into_rgba8();
        let (m, h) = img.dimensions();
        std::sync::Arc::new(egui::IconData {
            rgba: img.into_raw(),
            width: m,
            height: h,
        })
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(icon_data),
        ..Default::default()
    };
    eframe::run_native(
        "LTL Reader",
        options,
        Box::new(|cc| Ok(Box::new(LtlApp::new(cc, statuses)))),
    )
}
