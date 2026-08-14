use serde::{Deserialize, Serialize};
#[derive(Serialize,Deserialize)] #[serde(default)] pub struct Settings { pub dark: bool, pub diagnostics_height: f32 }
impl Default for Settings { fn default()->Self{Self{dark:true,diagnostics_height:150.0}} }
