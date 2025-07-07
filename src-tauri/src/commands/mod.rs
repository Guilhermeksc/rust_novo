// Módulos de comandos organizados por funcionalidade
pub mod pdf_commands;
pub mod config_commands;
pub mod directory_commands;
pub mod json_commands;
pub mod sicaf_commands;
pub mod file_operations;

// Re-exportar todos os comandos para uso fácil
pub use pdf_commands::*;
pub use config_commands::*;
pub use directory_commands::*;
pub use json_commands::*;
pub use sicaf_commands::*;
pub use file_operations::*;
