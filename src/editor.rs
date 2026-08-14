use cxxpect::{Diagnostic, SourceFile};
use eframe::egui::{self, text::{CCursor, CCursorRange, LayoutJob}, Color32, FontId, Stroke, TextFormat};

const KEYWORDS: &[&str] = &["contract","description","case","given","call","expect","returns","throws","nothing","oneof","message","contains","if","nothrow"];

pub struct EditorResult { pub changed: bool, pub cursor_line: usize, pub cursor_column: usize }

pub fn show(ui:&mut egui::Ui, text:&mut String, source:Option<&SourceFile>, diagnostics:&[Diagnostic], jump_to:&mut Option<usize>) -> EditorResult {
    let id=egui::Id::new("contract-editor");
    let mut cursor_line=1; let mut cursor_column=1;
    let output=egui::ScrollArea::both().id_salt("editor-scroll").show(ui,|ui|{
        ui.horizontal_top(|ui|{
            let lines=text.lines().count().max(1); let gutter=(1..=lines).map(|n|format!("{n:>4}\n")).collect::<String>();
            ui.add(egui::Label::new(egui::RichText::new(gutter).monospace().color(Color32::GRAY)).selectable(false));
            ui.separator();
            let diags=diagnostics.to_vec(); let src_text=source.map(|s|s.text().to_owned()).unwrap_or_else(||text.clone());
            let mut layouter=move |ui:&egui::Ui, buffer:&dyn egui::TextBuffer, wrap:f32| { let job=highlight(buffer.as_str(),wrap,&src_text,&diags); ui.fonts_mut(|f|f.layout_job(job)) };
            let mut edit=egui::TextEdit::multiline(text).id(id).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY).layouter(&mut layouter).lock_focus(true);
            edit=edit.code_editor();
            let mut out=edit.show(ui);
            if let Some(index)=jump_to.take(){ out.state.cursor.set_char_range(Some(CCursorRange::one(CCursor::new(index)))); out.state.store(ui.ctx(),id); out.response.request_focus(); }
            if let Some(range)=out.cursor_range { let index=range.primary.index; let prefix=&text[..byte_at_char(text,index)]; cursor_line=prefix.bytes().filter(|b|*b==b'\n').count()+1; cursor_column=prefix.rsplit('\n').next().unwrap_or("").chars().count()+1; }
            out.response.changed()
        }).inner
    }).inner;
    EditorResult{changed:output,cursor_line,cursor_column}
}

fn highlight(text:&str,wrap:f32,source_text:&str,diags:&[Diagnostic])->LayoutJob{
    let base=TextFormat{font_id:FontId::monospace(14.0),color:Color32::from_rgb(210,214,220),..Default::default()};
    let mut job=LayoutJob::default(); job.wrap.max_width=wrap;
    let mut byte=0;
    while byte<text.len(){ let rest=&text[byte..];
        let (end,color)=if rest.starts_with("//"){(rest.find('\n').unwrap_or(rest.len()),Color32::from_rgb(106,153,85))}
        else if rest.starts_with("/*"){(rest.find("*/").map(|n|n+2).unwrap_or(rest.len()),Color32::from_rgb(106,153,85))}
        else if rest.starts_with('"'){ let mut escaped=false; let end=rest.char_indices().skip(1).find_map(|(i,c)|{let close=c=='"'&&!escaped; escaped=c=='\\'&&!escaped; if c!='\\'{escaped=false;} close.then_some(i+c.len_utf8())}).unwrap_or(rest.len()); (end,Color32::from_rgb(206,145,120)) }
        else { let ch=rest.chars().next().unwrap(); if ch.is_alphabetic()||ch=='_' { let end=rest.char_indices().find(|(_,c)|!(c.is_alphanumeric()||*c=='_')).map(|(i,_)|i).unwrap_or(rest.len()); let word=&rest[..end]; (end,if KEYWORDS.contains(&word){Color32::from_rgb(86,156,214)}else{base.color}) } else {(ch.len_utf8(),base.color)} };
        let mut format=base.clone(); format.color=color;
        if diags.iter().any(|d| ranges_overlap(byte,byte+end,d.span.start.min(source_text.len()),d.span.end.max(d.span.start+1).min(source_text.len()))){format.underline=Stroke::new(1.0,Color32::from_rgb(244,71,71));}
        job.append(&rest[..end],0.0,format); byte+=end;
    } job
}
fn ranges_overlap(a:usize,b:usize,c:usize,d:usize)->bool{a<d&&c<b}
fn byte_at_char(s:&str,index:usize)->usize{s.char_indices().nth(index).map(|(i,_)|i).unwrap_or(s.len())}

#[cfg(test)] mod tests { use super::*; #[test] fn unicode_char_index_maps_to_bytes(){assert_eq!(byte_at_char("aЯb",2),3);} #[test] fn overlap_works(){assert!(ranges_overlap(2,5,4,8));assert!(!ranges_overlap(0,2,2,4));} }
