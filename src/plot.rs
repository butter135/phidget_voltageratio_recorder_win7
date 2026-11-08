use crate::model::Sample;
use std::collections::VecDeque;
use std::ptr;

use winapi::shared::windef::{HBRUSH, HDC, HGDIOBJ, POINT, RECT};

use winapi::um::wingdi::{
    CreatePen, DeleteObject, GetStockObject, IntersectClipRect, LineTo, MoveToEx, Polyline,
    RestoreDC, SaveDC, SelectObject, SetBkMode, SetTextAlign, SetTextColor, TextOutW, PS_SOLID,
    RGB, TA_CENTER, TA_LEFT, TA_RIGHT, TA_TOP, TRANSPARENT, WHITE_BRUSH,
};

use winapi::um::winuser::FillRect;

const WINDOW_SEC: f64 = 5.0;

// 描画領域の座標
struct PlotArea {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}
impl PlotArea {
    fn width(&self) -> i32 {
        self.right - self.left
    }

    fn height(&self) -> i32 {
        self.bottom - self.top
    }

    // VoltageRatio -> Y座標
    fn y(&self, ratio: f64) -> i32 {
        self.top + ((1.0 - ratio.clamp(0.0, 1.0)) * self.height() as f64) as i32
    }

    // 時刻 -> X座標
    fn x(&self, time: f64, begin: f64) -> i32 {
        self.left + (((time - begin) / WINDOW_SEC) * self.width() as f64) as i32
    }
}

// 文字描画
unsafe fn draw_text(hdc: HDC, x: i32, y: i32, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().collect();

    TextOutW(hdc, x, y, wide.as_ptr(), wide.len() as i32);
}

// 目盛りと補助線
unsafe fn draw_grid(hdc: HDC, area: &PlotArea) {
    let pen = CreatePen(PS_SOLID as i32, 1, RGB(220, 220, 220));

    if pen.is_null() {
        return;
    }

    let old_pen = SelectObject(hdc, pen as HGDIOBJ);

    // Y軸：1.0, 0.5, 0.0
    for i in 0..=2 {
        let ratio = 1.0 - i as f64 * 0.5;
        let y = area.y(ratio);

        MoveToEx(hdc, area.left, y, ptr::null_mut());
        LineTo(hdc, area.right, y);
    }

    // X軸：-5s ～ 0s
    for i in 0..=5 {
        let x = area.left + area.width() * i / 5;

        MoveToEx(hdc, x, area.top, ptr::null_mut());
        LineTo(hdc, x, area.bottom);
    }

    // GDIオブジェクトの後始末
    SelectObject(hdc, old_pen);
    DeleteObject(pen as HGDIOBJ);

    // 目盛り文字
    SetTextColor(hdc, RGB(90, 90, 90));
    SetBkMode(hdc, TRANSPARENT as i32);

    // Y軸ラベル
    SetTextAlign(hdc, TA_RIGHT | TA_TOP);

    for i in 0..=2 {
        let ratio = 1.0 - i as f64 * 0.5;
        let y = area.y(ratio);

        draw_text(hdc, area.left - 7, y - 8, &format!("{:.1}", ratio));
    }

    // X軸ラベル
    SetTextAlign(hdc, TA_CENTER | TA_TOP);

    for i in 0..=5 {
        let x = area.left + area.width() * i / 5;

        draw_text(hdc, x, area.bottom + 8, &format!("{}s", i - 5));
    }

    // 軸タイトル
    SetTextAlign(hdc, TA_LEFT | TA_TOP);
    draw_text(hdc, 7, 4, "Ratio");
}

// 波形の描画
unsafe fn draw_waveform(hdc: HDC, area: &PlotArea, samples: &VecDeque<Sample>) {
    let Some(last) = samples.back() else {
        return;
    };

    let begin = last.time - WINDOW_SEC;

    let points: Vec<POINT> = samples
        .iter()
        .filter(|s| s.time >= begin && s.time.is_finite() && s.voltage_ratio.is_finite())
        .map(|s| POINT {
            x: area.x(s.time, begin),
            y: area.y(s.voltage_ratio),
        })
        .collect();

    if points.len() < 2 {
        return;
    }

    // 描画範囲をプロット領域に制限
    let saved = SaveDC(hdc);

    if saved == 0 {
        return;
    }

    IntersectClipRect(hdc, area.left, area.top, area.right + 1, area.bottom + 1);

    let pen = CreatePen(PS_SOLID as i32, 2, RGB(30, 120, 215));

    if !pen.is_null() {
        SelectObject(hdc, pen as HGDIOBJ);

        Polyline(hdc, points.as_ptr(), points.len() as i32);
    }

    // 元の描画状態を復元してからペンを解放
    RestoreDC(hdc, saved);

    if !pen.is_null() {
        DeleteObject(pen as HGDIOBJ);
    }
}

// GUIから呼び出す描画関数
pub fn draw(hdc: HDC, width: i32, height: i32, samples: &VecDeque<Sample>) {
    if width < 120 || height < 100 {
        return;
    }

    let area = PlotArea {
        left: 47,
        right: width - 20,
        top: 29,
        bottom: height - 36,
    };

    unsafe {
        let saved = SaveDC(hdc);

        if saved == 0 {
            return;
        }

        // 背景を白で塗りつぶす
        let rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };

        let brush = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;

        FillRect(hdc, &rect, brush);

        // 目盛りと波形
        draw_grid(hdc, &area);
        draw_waveform(hdc, &area, samples);

        RestoreDC(hdc, saved);
    }
}
