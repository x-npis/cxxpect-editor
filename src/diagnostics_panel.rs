use cxxpect::{Diagnostic, SourceFile};
use eframe::egui;

pub fn show(ui:&mut egui::Ui,source:Option<&SourceFile>,diagnostics:&[Diagnostic])->Option<usize>{
    let mut jump=None;
    ui.horizontal(|ui|{ui.strong(format!("Diagnostics ({})",diagnostics.len()));}); ui.separator();
    egui::ScrollArea::vertical().show(ui,|ui| for d in diagnostics { let range=source.map(|s|s.locate(d.span)); let position=range.map(|r|format!("{}:{}",r.start.line,r.start.column)).unwrap_or_default(); if ui.selectable_label(false,format!("{}  {}  {}",d.code,position,d.message)).on_hover_text(d.note.as_deref().unwrap_or(&d.message)).clicked(){jump=range.map(|r|r.start.char_offset);} });
    jump
}
