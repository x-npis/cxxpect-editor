use cxxpect::{Diagnostic, SourceFile, verify};
use std::{path::PathBuf, sync::mpsc::{self, Receiver, Sender}, thread, time::{Duration, Instant}};

pub const DEBOUNCE: Duration = Duration::from_millis(300);
struct Request { revision: u64, path: PathBuf, text: String }
pub struct ResultSet { pub revision: u64, pub source: SourceFile, pub diagnostics: Vec<Diagnostic> }

pub struct Verifier { tx: Sender<Request>, rx: Receiver<ResultSet>, pending: Option<(u64, Instant)>, pub current: Option<ResultSet>, pub running: bool }
impl Verifier {
    pub fn new() -> Self {
        let (tx, work_rx)=mpsc::channel::<Request>(); let (result_tx,rx)=mpsc::channel();
        thread::spawn(move || while let Ok(r)=work_rx.recv() { let source=SourceFile::new(r.path,r.text); let report=verify(&source); let _=result_tx.send(ResultSet{revision:r.revision,source,diagnostics:report.diagnostics}); });
        Self { tx,rx,pending:None,current:None,running:false }
    }
    pub fn schedule(&mut self, revision:u64){ self.pending=Some((revision,Instant::now())); }
    pub fn update(&mut self, revision:u64, path:PathBuf, text:&str) {
        if self.pending.is_some_and(|(r,t)| r==revision && t.elapsed()>=DEBOUNCE) { self.pending=None; self.running=self.tx.send(Request{revision,path,text:text.into()}).is_ok(); }
        while let Ok(result)=self.rx.try_recv(){ if result.revision==revision { self.current=Some(result); self.running=false; } }
    }
    pub fn verify_now(&mut self, revision:u64,path:PathBuf,text:&str){ self.pending=None; self.running=self.tx.send(Request{revision,path,text:text.into()}).is_ok(); }
}

#[cfg(test)] mod tests { use super::*; #[test] fn debounce_is_in_required_range(){assert!(DEBOUNCE>=Duration::from_millis(250));assert!(DEBOUNCE<=Duration::from_millis(350));} }
