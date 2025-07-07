use std::path::PathBuf;
use walkdir::WalkDir;
use crate::types::TauriError;

/// Obtém informações de um arquivo PDF específico
#[tauri::command]
pub async fn get_pdf_file_info(file_path: String) -> Result<serde_json::Value, TauriError> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path.clone()),
        });
    }
    
    let metadata = std::fs::metadata(&path).map_err(|e| TauriError {
        error_type: "FileSystemError".to_string(),
        message: format!("Erro ao ler metadados do arquivo: {}", e),
        details: Some(file_path.clone()),
    })?;
    
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let file_size = metadata.len();
    let modified = metadata.modified()
        .map_err(|e| TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao ler data de modificação: {}", e),
            details: Some(file_path.clone()),
        })?;
    
    let modified_timestamp = modified.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let file_info = serde_json::json!({
        "file_name": file_name,
        "file_path": file_path,
        "file_size": file_size,
        "modified_timestamp": modified_timestamp
    });
    
    Ok(file_info)
}

/// Obtém informações de todos os arquivos PDF em um diretório
#[tauri::command]
pub async fn get_pdf_files_info(directory: String) -> Result<Vec<serde_json::Value>, TauriError> {
    let path = PathBuf::from(&directory);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Diretório não encontrado: {}", directory),
            details: Some(directory),
        });
    }
    
    let mut pdf_files_info = Vec::new();
    
    for entry in WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
    {
        let file_path = entry.path().to_string_lossy().to_string();
        
        if let Ok(metadata) = entry.metadata() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_size = metadata.len();
            
            let modified_timestamp = metadata.modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            
            let file_info = serde_json::json!({
                "file_name": file_name,
                "file_path": file_path,
                "file_size": file_size,
                "modified_timestamp": modified_timestamp
            });
            
            pdf_files_info.push(file_info);
        }
    }
    
    // Ordenar por data de modificação (mais recente primeiro)
    pdf_files_info.sort_by(|a, b| {
        let a_timestamp = a["modified_timestamp"].as_u64().unwrap_or(0);
        let b_timestamp = b["modified_timestamp"].as_u64().unwrap_or(0);
        b_timestamp.cmp(&a_timestamp)
    });
    
    Ok(pdf_files_info)
}

/// Abre um arquivo PDF no visualizador padrão do sistema
#[tauri::command]
pub async fn open_pdf_file(file_path: String) -> Result<bool, TauriError> {
    let path_buf = PathBuf::from(&file_path);
    
    // Verificar se o arquivo existe
    if !path_buf.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path.clone()),
        });
    }
    
    // Verificar se é um arquivo PDF
    if path_buf.extension().map_or(true, |ext| ext != "pdf") {
        return Err(TauriError {
            error_type: "ValidationError".to_string(),
            message: "O arquivo deve ter extensão .pdf".to_string(),
            details: Some(file_path.clone()),
        });
    }
    
    // Abrir arquivo no sistema operacional
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &file_path])
            .spawn()
            .map_err(|e| TauriError {
                error_type: "SystemError".to_string(),
                message: format!("Erro ao abrir arquivo PDF: {}", e),
                details: Some(file_path.clone()),
            })?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| TauriError {
                error_type: "SystemError".to_string(),
                message: format!("Erro ao abrir arquivo PDF: {}", e),
                details: Some(file_path.clone()),
            })?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| TauriError {
                error_type: "SystemError".to_string(),
                message: format!("Erro ao abrir arquivo PDF: {}", e),
                details: Some(file_path.clone()),
            })?;
    }
    
    Ok(true)
}
