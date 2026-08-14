use crate::{commands::Command, diagnostics_panel, document::Document, editor, settings::Settings, verification::Verifier};
use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use std::path::PathBuf;

#[derive(Clone)] enum Pending { New, Open, OpenPath(PathBuf), Quit }
pub struct EditorApp { document:Document, verifier:Verifier, settings:Settings, cursor:(usize,usize), jump_to:Option<usize>, find_open:bool, find:String, replace:String, confirm:Option<Pending>, error:Option<String>, diagnostic_index:usize }

impl EditorApp {
    pub fn new(cc:&eframe::CreationContext<'_>)->Self{
        let settings=cc.storage.and_then(|s|eframe::get_value(s,"settings")).unwrap_or_default();
        if settings.dark {cc.egui_ctx.set_visuals(egui::Visuals::dark())} else {cc.egui_ctx.set_visuals(egui::Visuals::light())}
        let document=Document::new(); let mut verifier=Verifier::new(); verifier.verify_now(0,PathBuf::from("Untitled.cxxp"),&document.text);
        Self{document,verifier,settings,cursor:(1,1),jump_to:None,find_open:false,find:String::new(),replace:String::new(),confirm:None,error:None,diagnostic_index:0}
    }
    fn dispatch(&mut self,cmd:Command,ctx:&egui::Context){match cmd{
        Command::New=>self.guard(Pending::New), Command::Open=>self.guard(Pending::Open), Command::Quit=>{if self.document.is_dirty(){self.confirm=Some(Pending::Quit)}else{ctx.send_viewport_cmd(egui::ViewportCommand::Close)}},
        Command::Save=>{self.save(false);}, Command::SaveAs=>{self.save(true);}, Command::Find=>self.find_open=true,
        Command::Undo=>send_edit_key(ctx,Key::Z,false), Command::Redo=>send_edit_key(ctx,Key::Z,true), Command::NextDiagnostic=>self.next_diagnostic(),
        Command::ToggleTheme=>{self.settings.dark=!self.settings.dark;ctx.set_visuals(if self.settings.dark{egui::Visuals::dark()}else{egui::Visuals::light()});}
    }}
    fn guard(&mut self,p:Pending){if self.document.is_dirty(){self.confirm=Some(p)}else{self.complete(p)}}
    fn complete(&mut self,p:Pending){match p{Pending::New=>{self.document=Document::new();self.changed();},Pending::Open=>self.open_dialog(),Pending::OpenPath(path)=>match Document::open(path){Ok(d)=>{self.document=d;self.changed()},Err(e)=>self.error=Some(e.to_string())},Pending::Quit=>{}}}
    fn open_dialog(&mut self){if let Some(path)=rfd::FileDialog::new().add_filter("Cxxpect contract", &["cxxp"]).pick_file(){match Document::open(path){Ok(d)=>{self.document=d;self.changed()},Err(e)=>self.error=Some(e.to_string())}}}
    fn save(&mut self,force_dialog:bool)->bool{let path=if !force_dialog{self.document.path.clone()}else{None}.or_else(||rfd::FileDialog::new().add_filter("Cxxpect contract", &["cxxp"]).set_file_name(&self.document.title()).save_file());match path{Some(p)=>match self.document.save_to(p){Ok(())=>true,Err(e)=>{self.error=Some(e.to_string());false}},None=>false}}
    fn changed(&mut self){self.document.changed();self.verifier.schedule(self.document.revision);}
    fn next_diagnostic(&mut self){if let Some(current)=&self.verifier.current{if !current.diagnostics.is_empty(){self.diagnostic_index=(self.diagnostic_index+1)%current.diagnostics.len();self.jump_to=Some(current.source.locate(current.diagnostics[self.diagnostic_index].span).start.char_offset);}}}
    fn shortcuts(&mut self,ctx:&egui::Context){let take=|key,shift|ctx.input_mut(|i|i.consume_shortcut(&KeyboardShortcut::new(Modifiers{command:true,shift,..Default::default()},key)));if take(Key::N,false){self.dispatch(Command::New,ctx)}if take(Key::O,false){self.dispatch(Command::Open,ctx)}if take(Key::S,false){self.dispatch(Command::Save,ctx)}if take(Key::S,true){self.dispatch(Command::SaveAs,ctx)}if take(Key::F,false){self.dispatch(Command::Find,ctx)}if ctx.input_mut(|i|i.consume_key(Modifiers::NONE,Key::F8)){self.dispatch(Command::NextDiagnostic,ctx)}}
    fn find_replace(&mut self,ctx:&egui::Context){if !self.find_open{return}egui::Window::new("Find and replace").collapsible(false).resizable(false).show(ctx,|ui|{ui.horizontal(|ui|{ui.label("Find");ui.text_edit_singleline(&mut self.find);});ui.horizontal(|ui|{ui.label("Replace");ui.text_edit_singleline(&mut self.replace);});ui.horizontal(|ui|{if ui.button("Find next").clicked()&&!self.find.is_empty(){let start=char_to_byte(&self.document.text,self.jump_to.unwrap_or(0));let found=self.document.text[start..].find(&self.find).map(|p|start+p).or_else(||self.document.text[..start].find(&self.find));if let Some(pos)=found{self.jump_to=Some(self.document.text[..pos].chars().count());}}if ui.button("Replace all").clicked()&&!self.find.is_empty(){self.document.text=self.document.text.replace(&self.find,&self.replace);self.changed();}if ui.button("Close").clicked(){self.find_open=false;}});});}
}

impl eframe::App for EditorApp {
 fn save(&mut self,storage:&mut dyn eframe::Storage){eframe::set_value(storage,"settings",&self.settings)}
 fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame){
  self.shortcuts(ctx); let path=self.document.path.clone().unwrap_or_else(||PathBuf::from("Untitled.cxxp"));self.verifier.update(self.document.revision,path,&self.document.text);if self.verifier.running{ctx.request_repaint_after(std::time::Duration::from_millis(30));}
  for file in ctx.input(|i|i.raw.dropped_files.clone()){if let Some(path)=file.path.filter(|p|p.extension().is_some_and(|e|e.to_string_lossy().eq_ignore_ascii_case("cxxp"))){if self.document.is_dirty(){self.confirm=Some(Pending::OpenPath(path))}else if let Ok(d)=Document::open(path){self.document=d;self.changed();}}}
  if ctx.input(|i|i.viewport().close_requested()){if self.document.is_dirty(){ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);self.confirm=Some(Pending::Quit);}}
  egui::TopBottomPanel::top("menu").show(ctx,|ui|{egui::MenuBar::new().ui(ui,|ui|{ui.menu_button("File",|ui|{for (name,c) in [("New",Command::New),("Open...",Command::Open),("Save",Command::Save),("Save as...",Command::SaveAs),("Quit",Command::Quit)]{if ui.button(name).clicked(){self.dispatch(c,ctx);ui.close();}}});ui.menu_button("Edit",|ui|{for (name,c) in [("Undo",Command::Undo),("Redo",Command::Redo),("Find / Replace",Command::Find)]{if ui.button(name).clicked(){self.dispatch(c,ctx);ui.close();}}});ui.menu_button("View",|ui|{if ui.button("Toggle theme").clicked(){self.dispatch(Command::ToggleTheme,ctx);ui.close();}});ui.separator();ui.label(format!("{}{}",self.document.title(),if self.document.is_dirty(){" *"}else{""}));});ui.separator();ui.horizontal(|ui|{for (icon,tip,c) in [("＋","New",Command::New),("◫","Open",Command::Open),("▣","Save",Command::Save),("↶","Undo",Command::Undo),("↷","Redo",Command::Redo),("⌕","Find / Replace",Command::Find),("▶","Next diagnostic",Command::NextDiagnostic)]{if ui.small_button(icon).on_hover_text(tip).clicked(){self.dispatch(c,ctx);}}});});
  egui::TopBottomPanel::bottom("status").exact_height(24.0).show(ctx,|ui|{let count=self.verifier.current.as_ref().map(|r|r.diagnostics.len()).unwrap_or(0);ui.horizontal(|ui|{ui.label(format!("Ln {}, Col {}",self.cursor.0,self.cursor.1));ui.separator();ui.label(if self.verifier.running{"Checking..."}else if count==0{"No errors"}else{"Errors"});ui.separator();ui.label(format!("{count} diagnostic(s)"));ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(format!("UTF-8 · {}",self.document.line_ending));});});});
  let (source,diagnostics)=self.verifier.current.as_ref().filter(|r|r.revision==self.document.revision).map(|r|(Some(r.source.clone()),r.diagnostics.clone())).unwrap_or((None,Vec::new()));
  let panel=egui::TopBottomPanel::bottom("diagnostics").resizable(true).default_height(self.settings.diagnostics_height).height_range(90.0..=300.0).show(ctx,|ui|{if let Some(index)=diagnostics_panel::show(ui,source.as_ref(),&diagnostics){self.jump_to=Some(index);}});self.settings.diagnostics_height=panel.response.rect.height();
  egui::CentralPanel::default().show(ctx,|ui|{let result=editor::show(ui,&mut self.document.text,source.as_ref(),&diagnostics,&mut self.jump_to);self.cursor=(result.cursor_line,result.cursor_column);if result.changed{self.changed();}});
  self.find_replace(ctx); self.modals(ctx);
 }
}

impl EditorApp { fn modals(&mut self,ctx:&egui::Context){if let Some(p)=self.confirm.clone(){egui::Window::new("Unsaved changes").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER,[0.0,0.0]).show(ctx,|ui|{ui.label("Save changes before continuing?");ui.horizontal(|ui|{if ui.button("Save").clicked()&&self.save(false){self.confirm=None;if matches!(&p,Pending::Quit){ctx.send_viewport_cmd(egui::ViewportCommand::Close)}else{self.complete(p.clone())}}if ui.button("Discard").clicked(){self.confirm=None;if matches!(&p,Pending::Quit){ctx.send_viewport_cmd(egui::ViewportCommand::Close)}else{self.complete(p.clone())}}if ui.button("Cancel").clicked(){self.confirm=None;}});});}if let Some(message)=self.error.clone(){egui::Window::new("Error").collapsible(false).resizable(false).show(ctx,|ui|{ui.label(message);if ui.button("OK").clicked(){self.error=None;}});}}}
fn send_edit_key(ctx:&egui::Context,key:Key,shift:bool){ctx.memory_mut(|m|m.request_focus(egui::Id::new("contract-editor")));ctx.input_mut(|i|i.events.push(egui::Event::Key{key,physical_key:None,pressed:true,repeat:false,modifiers:Modifiers{command:true,shift,..Default::default()}}));}
fn char_to_byte(s:&str,index:usize)->usize{s.char_indices().nth(index).map(|v|v.0).unwrap_or(s.len())}
