#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Entry point and window/font setup. Application logic lives in the modules
// below.

mod app;
mod model;
mod srs;
mod ui;
mod util;

use std::fs;

use eframe::egui;

use crate::app::App;

/// Candidate CJK font files to try, per platform. The first one that exists is
/// used so Chinese text renders correctly instead of showing empty boxes.
fn cjk_font_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &[
            r"C:\Windows\Fonts\msyh.ttc",   // Microsoft YaHei
            r"C:\Windows\Fonts\simhei.ttf", // SimHei
            r"C:\Windows\Fonts\simsun.ttc", // SimSun
        ]
    }
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/PingFang.ttc", // PingFang SC (modern default)
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
        ]
    }
}

/// Load a CJK-capable font so Chinese text renders correctly.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in cjk_font_candidates() {
        if let Ok(data) = fs::read(path) {
            fonts
                .font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(data));
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "cjk".to_owned());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push("cjk".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([720.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Deutsch Worttrainer",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_pixels_per_point(1.15);
            Ok(Box::new(App::new()))
        }),
    )
}
