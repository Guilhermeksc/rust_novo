use std::path::PathBuf;
use walkdir::WalkDir;
use crate::types::TauriError;

/// Lista arquivos JSON em um diretório
#[tauri::command]
pub async fn list_json_files(directory: String) -> Result<Vec<String>, TauriError> {
    let path = PathBuf::from(&directory);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Diretório não encontrado: {}", directory),
            details: Some(directory),
        });
    }
    
    let mut json_files = Vec::new();
    
    for entry in WalkDir::new(&path)
        .max_depth(2) // Limitar profundidade para evitar muitos arquivos
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
    {
        json_files.push(entry.path().to_string_lossy().to_string());
    }
    
    // Ordenar por data de modificação (mais recente primeiro)
    json_files.sort_by(|a, b| {
        let metadata_a = std::fs::metadata(a).ok();
        let metadata_b = std::fs::metadata(b).ok();
        
        match (metadata_a, metadata_b) {
            (Some(meta_a), Some(meta_b)) => {
                meta_b.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    .cmp(&meta_a.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))
            }
            _ => std::cmp::Ordering::Equal
        }
    });
    
    Ok(json_files)
}

/// Lê e retorna o conteúdo de um arquivo JSON
#[tauri::command]
pub async fn read_json_file(file_path: String) -> Result<serde_json::Value, TauriError> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path),
        });
    }
    
    if path.extension().map_or(true, |ext| ext != "json") {
        return Err(TauriError {
            error_type: "ValidationError".to_string(),
            message: "O arquivo deve ter extensão .json".to_string(),
            details: Some(file_path),
        });
    }
    
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => Ok(json),
                Err(e) => Err(TauriError {
                    error_type: "ParseError".to_string(),
                    message: format!("Erro ao analisar JSON: {}", e),
                    details: Some(file_path),
                })
            }
        }
        Err(e) => Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao ler arquivo: {}", e),
            details: Some(file_path),
        })
    }
}

/// Obtém informações detalhadas de um arquivo JSON
#[tauri::command]
pub async fn get_json_file_info(file_path: String) -> Result<serde_json::Value, TauriError> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path.clone()),
        });
    }
    
    // Obter metadados do arquivo
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
    
    // Tentar ler e analisar o conteúdo JSON
    let mut file_info = serde_json::json!({
        "file_name": file_name,
        "file_path": file_path,
        "file_size": file_size,
        "modified_timestamp": modified_timestamp
    });
    
    // Tentar extrair informações específicas do JSON
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    // Extrair informações específicas se disponíveis
                    if let Some(data_geracao) = json.get("data_geracao").and_then(|v| v.as_str()) {
                        file_info["data_geracao"] = serde_json::Value::String(data_geracao.to_string());
                    }
                    
                    if let Some(pregao) = json.get("pregao").and_then(|v| v.as_str()) {
                        file_info["pregao"] = serde_json::Value::String(pregao.to_string());
                    }
                    
                    if let Some(processo) = json.get("processo").and_then(|v| v.as_str()) {
                        file_info["processo"] = serde_json::Value::String(processo.to_string());
                    }
                    
                    if let Some(uasg) = json.get("uasg").and_then(|v| v.as_str()) {
                        file_info["uasg"] = serde_json::Value::String(uasg.to_string());
                    }
                    
                    if let Some(total_propostas) = json.get("total_propostas").and_then(|v| v.as_u64()) {
                        file_info["total_propostas"] = serde_json::Value::Number(serde_json::Number::from(total_propostas));
                    }
                    
                    if let Some(valor_total) = json.get("valor_total").and_then(|v| v.as_f64()) {
                        file_info["valor_total"] = serde_json::Value::Number(serde_json::Number::from_f64(valor_total).unwrap_or(serde_json::Number::from(0)));
                    }
                    
                    // Contar propostas se for um array
                    if let Some(propostas) = json.get("propostas").and_then(|v| v.as_array()) {
                        file_info["propostas_count"] = serde_json::Value::Number(serde_json::Number::from(propostas.len()));
                    }
                }
                Err(e) => {
                    file_info["error"] = serde_json::Value::String(format!("Erro ao analisar JSON: {}", e));
                }
            }
        }
        Err(e) => {
            file_info["error"] = serde_json::Value::String(format!("Erro ao ler arquivo: {}", e));
        }
    }
    
    Ok(file_info)
}
