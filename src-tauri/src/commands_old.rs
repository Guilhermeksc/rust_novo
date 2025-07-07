use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::State;
use crate::types::*;
use crate::pdf_processor;
use crate::sicaf_processor;
use crate::config;
use walkdir::WalkDir;
use chrono::Utc;

// Estado global para rastrear o progresso do processamento
pub type ProcessingState = Arc<Mutex<HashMap<String, ProcessingStatus>>>;

/// Processa um único arquivo PDF
#[tauri::command]
pub async fn process_pdf_file(
    file_path: String,
    output_dir: String,
    verbose: bool,
    processing_state: State<'_, ProcessingState>
) -> Result<ProcessingResult, TauriError> {
    let session_id = format!("pdf_file_{}", Utc::now().timestamp_millis());
    let input_path = PathBuf::from(&file_path);
    let output_path = PathBuf::from(&output_dir);
    
    // Verificar se o arquivo existe
    if !input_path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path.clone()),
        });
    }
    
    // Verificar se é um arquivo PDF
    if input_path.extension().map_or(true, |ext| ext != "pdf") {
        return Err(TauriError {
            error_type: "ValidationError".to_string(),
            message: "O arquivo deve ter extensão .pdf".to_string(),
            details: Some(file_path.clone()),
        });
    }
    
    // Criar diretório de saída se não existir
    if let Err(e) = std::fs::create_dir_all(&output_path) {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao criar diretório de saída: {}", e),
            details: Some(output_dir.clone()),
        });
    }
    
    // Inicializar estado de processamento
    {
        let mut state = processing_state.lock().unwrap();
        state.insert(session_id.clone(), ProcessingStatus {
            is_processing: true,
            current_file: Some(file_path.clone()),
            processed_files: 0,
            total_files: 1,
            errors: Vec::new(),
            progress_percentage: 0.0,
        });
    }
    
    match pdf_processor::processar_pdf_com_consolidacao(&input_path, &output_path, verbose) {
        Ok(propostas) => {
            // Atualizar progresso final
            {
                let mut state = processing_state.lock().unwrap();
                if let Some(status) = state.get_mut(&session_id) {
                    status.processed_files = 1;
                    status.progress_percentage = 100.0;
                    status.is_processing = false;
                }
            }
            
            // Gerar nome do arquivo de saída baseado no arquivo de entrada
            let file_stem = input_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let json_file_path = output_path.join(format!("{}.json", file_stem));
            
            Ok(ProcessingResult {
                success: true,
                message: format!("Arquivo processado com sucesso: {} propostas encontradas", propostas.len()),
                propostas,
                total_processed: 1,
                json_file_path: Some(json_file_path.to_string_lossy().to_string()),
                session_id: Some(session_id),
            })
        }
        Err(e) => {
            // Atualizar estado com erro
            {
                let mut state = processing_state.lock().unwrap();
                if let Some(status) = state.get_mut(&session_id) {
                    status.is_processing = false;
                    status.errors.push(format!("Erro ao processar arquivo: {}", e));
                }
            }
            
            Err(TauriError {
                error_type: "ProcessingError".to_string(),
                message: format!("Erro ao processar arquivo: {}", e),
                details: Some(file_path),
            })
        }
    }
}

/// Processa múltiplos arquivos PDF em um diretório
#[tauri::command]
pub async fn process_pdf_directory(
    input_dir: String,
    output_dir: String,
    verbose: bool,
    session_id: Option<String>,
    processing_state: State<'_, ProcessingState>
) -> Result<ProcessingResult, TauriError> {
    let session_id = session_id.unwrap_or_else(|| format!("pdf_directory_{}", Utc::now().timestamp_millis()));
    
    let input_path = PathBuf::from(&input_dir);
    let output_path = PathBuf::from(&output_dir);
    
    // Verificar se o diretório de entrada existe
    if !input_path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Diretório de entrada não encontrado: {}", input_dir),
            details: Some(input_dir.clone()),
        });
    }
    
    // Contar arquivos PDF no diretório
    let total_files = WalkDir::new(&input_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
        .count();
    
    if total_files == 0 {
        return Err(TauriError {
            error_type: "ValidationError".to_string(),
            message: "Nenhum arquivo PDF encontrado no diretório especificado".to_string(),
            details: Some(input_dir.clone()),
        });
    }
    
    // Inicializar estado de processamento
    {
        let mut state = processing_state.lock().unwrap();
        state.insert(session_id.clone(), ProcessingStatus {
            is_processing: true,
            current_file: None,
            processed_files: 0,
            total_files,
            errors: Vec::new(),
            progress_percentage: 0.0,
        });
    }
    
    // Processar todos os arquivos
    let processing_state_clone = processing_state.clone();
    let session_id_clone = session_id.clone();
    
    match pdf_processor::processar_diretorio_pdfs_com_progresso(
        &input_path, 
        &output_path, 
        verbose,
        |processed, total, current_file| {
            // Atualizar progresso em tempo real
            let mut state = processing_state_clone.lock().unwrap();
            if let Some(status) = state.get_mut(&session_id_clone) {
                status.processed_files = processed;
                status.total_files = total;
                status.current_file = current_file;
                status.progress_percentage = if total > 0 { (processed as f64 / total as f64) * 100.0 } else { 0.0 };
            }
        }
    ) {
        Ok(propostas) => {
            // Atualizar progresso final
            {
                let mut state = processing_state.lock().unwrap();
                if let Some(status) = state.get_mut(&session_id) {
                    status.processed_files = total_files;
                    status.progress_percentage = 100.0;
                    status.is_processing = false;
                }
            }
            
            // Salvar JSON consolidado
            if let Err(e) = pdf_processor::salvar_json_consolidado(&propostas, &output_path, "consolidado.json", verbose) {
                let error = TauriError {
                    error_type: "ProcessingError".to_string(),
                    message: format!("Erro ao salvar JSON consolidado: {}", e),
                    details: Some(input_dir.clone()),
                };
                
                {
                    let mut state = processing_state.lock().unwrap();
                    if let Some(status) = state.get_mut(&session_id) {
                        status.errors.push(error.message.clone());
                    }
                }
                
                return Err(error);
            }
            
            let json_file_path = output_path.join("resumo_geral.json");
            
            Ok(ProcessingResult {
                success: true,
                message: format!("Diretório processado com sucesso: {} arquivos processados, {} propostas encontradas", total_files, propostas.len()),
                propostas,
                total_processed: total_files,
                json_file_path: Some(json_file_path.to_string_lossy().to_string()),
                session_id: Some(session_id),
            })
        }
        Err(e) => {
            let error = TauriError {
                error_type: "ProcessingError".to_string(),
                message: format!("Erro ao processar diretório: {}", e),
                details: Some(input_dir.clone()),
            };
            
            // Atualizar estado com erro
            {
                let mut state = processing_state.lock().unwrap();
                if let Some(status) = state.get_mut(&session_id) {
                    status.is_processing = false;
                    status.errors.push(error.message.clone());
                }
            }
            
            Err(error)
        }
    }
}

/// Obtém o status atual do processamento
#[tauri::command]
pub async fn get_processing_status(
    session_id: String,
    processing_state: State<'_, ProcessingState>
) -> Result<ProcessingStatus, TauriError> {
    let state = processing_state.lock().unwrap();
    
    match state.get(&session_id) {
        Some(status) => Ok(status.clone()),
        None => Err(TauriError {
            error_type: "SessionNotFound".to_string(),
            message: format!("Sessão de processamento não encontrada: {}", session_id),
            details: Some(session_id),
        })
    }
}

/// Lista arquivos PDF em um diretório
#[tauri::command]
pub async fn list_pdf_files(directory: String) -> Result<Vec<String>, TauriError> {
    let path = PathBuf::from(&directory);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Diretório não encontrado: {}", directory),
            details: Some(directory),
        });
    }
    
    let mut pdf_files = Vec::new();
    
    for entry in WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
    {
        pdf_files.push(entry.path().to_string_lossy().to_string());
    }
    
    Ok(pdf_files)
}

/// Valida se um arquivo PDF é válido
#[tauri::command]
pub async fn validate_pdf_file(file_path: String) -> Result<bool, TauriError> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path),
        });
    }
    
    // Verificar se é um arquivo PDF
    if path.extension().map_or(false, |ext| ext == "pdf") {
        // Tentar extrair texto para validar se é um PDF válido
        match pdf_extract::extract_text(&path) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    } else {
        Ok(false)
    }
}

/// Limpa o estado de processamento (útil para limpeza)
#[tauri::command]
pub async fn clear_processing_state(
    session_id: String,
    processing_state: State<'_, ProcessingState>
) -> Result<(), TauriError> {
    let mut state = processing_state.lock().unwrap();
    state.remove(&session_id);
    Ok(())
}

/// Obtém o diretório de trabalho atual
#[tauri::command]
pub async fn get_current_directory() -> Result<String, TauriError> {
    match std::env::current_dir() {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao obter diretório atual: {}", e),
            details: None,
        })
    }
}

/// Cria as pastas padrão se não existirem
#[tauri::command]
pub async fn create_default_directories() -> Result<String, TauriError> {
    // Usar as funções específicas para garantir consistência
    let pdf_dir = get_pdf_directory().await?;
    let output_dir = get_output_directory().await?;
    let sicaf_dir = get_sicaf_directory().await?;
    
    Ok(format!("Estrutura Database criada:\n- PDFs: {}\n- Resultados: {}\n- SICAF: {}", 
        pdf_dir, output_dir, sicaf_dir))
}

/// Inicializa toda a estrutura de pastas Database
#[tauri::command]
pub async fn initialize_database_structure() -> Result<String, TauriError> {
    let current_exe = std::env::current_exe()
        .map_err(|e| TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao obter diretório do executável: {}", e),
            details: None,
        })?;
    
    let exe_dir = current_exe.parent()
        .ok_or_else(|| TauriError {
            error_type: "FileSystemError".to_string(),
            message: "Não foi possível obter o diretório pai do executável".to_string(),
            details: None,
        })?;
    
    let database_dir = exe_dir.join("Database");
    let subdirs = ["PDFs", "Resultados", "SICAF", "Config"];
    
    // Criar pasta Database principal
    if !database_dir.exists() {
        std::fs::create_dir_all(&database_dir)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar pasta Database: {}", e),
                details: Some(database_dir.to_string_lossy().to_string()),
            })?;
    }
    
    // Criar subpastas
    for subdir in &subdirs {
        let dir_path = database_dir.join(subdir);
        if !dir_path.exists() {
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| TauriError {
                    error_type: "FileSystemError".to_string(),
                    message: format!("Erro ao criar pasta {}: {}", subdir, e),
                    details: Some(dir_path.to_string_lossy().to_string()),
                })?;
        }
    }
    
    // Criar arquivo README na pasta Database
    let readme_path = database_dir.join("README.txt");
    if !readme_path.exists() {
        let readme_content = r#"=== LICITAÇÃO 360 - ESTRUTURA DE PASTAS ===

Esta pasta contém todos os dados do sistema:

📁 PDFs/       - Arquivos PDF de licitações para processamento
📁 Resultados/ - Arquivos JSON processados das licitações  
📁 SICAF/      - Arquivos PDF do SICAF para verificação
📁 Config/     - Configurações do sistema

IMPORTANTE: NÃO delete esta pasta! Ela contém todos os seus dados.

Esta estrutura é mantida durante atualizações do programa.
"#;
        
        std::fs::write(&readme_path, readme_content)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar README: {}", e),
                details: Some(readme_path.to_string_lossy().to_string()),
            })?;
    }
    
    Ok(format!("Estrutura Database inicializada com sucesso em: {}", database_dir.to_string_lossy()))
}

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

/// Obtém informações resumidas de um arquivo JSON sem carregar todo o conteúdo
#[tauri::command]
pub async fn get_json_file_info(file_path: String) -> Result<serde_json::Value, TauriError> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Arquivo não encontrado: {}", file_path),
            details: Some(file_path),
        });
    }
    
    let metadata = std::fs::metadata(&path).map_err(|e| TauriError {
        error_type: "FileSystemError".to_string(),
        message: format!("Erro ao obter informações do arquivo: {}", e),
        details: Some(file_path.clone()),
    })?;
    
    let file_name = path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    
    let file_size = metadata.len();
    let modified = metadata.modified()
        .map(|time| time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);
    
    // Tentar ler informações básicas do JSON sem carregar propostas
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return Ok(serde_json::json!({
                    "file_name": file_name,
                    "file_path": file_path,
                    "file_size": file_size,
                    "modified_timestamp": modified,
                    "error": "Arquivo JSON está vazio"
                }));
            }
            
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    let mut info = serde_json::json!({
                        "file_name": file_name,
                        "file_path": file_path,
                        "file_size": file_size,
                        "modified_timestamp": modified,
                        "data_geracao": json.get("data_geracao").cloned().unwrap_or(serde_json::Value::Null),
                        "pregao": json.get("pregao").cloned().unwrap_or(serde_json::Value::Null),
                        "processo": json.get("processo").cloned().unwrap_or(serde_json::Value::Null),
                        "uasg": json.get("uasg").cloned().unwrap_or(serde_json::Value::Null),
                        "total_propostas": json.get("total_propostas").cloned().unwrap_or(serde_json::Value::Null),
                        "valor_total": json.get("valor_total").cloned().unwrap_or(serde_json::Value::Null)
                    });
                    
                    // Contar propostas se existir array
                    if let Some(propostas) = json.get("propostas").and_then(|p| p.as_array()) {
                        info["propostas_count"] = serde_json::Value::from(propostas.len());
                    } else {
                        info["propostas_count"] = serde_json::Value::from(0);
                    }
                    
                    Ok(info)
                }
                Err(e) => {
                    // Se não conseguir ler JSON, retorna apenas info básica do arquivo
                    Ok(serde_json::json!({
                        "file_name": file_name,
                        "file_path": file_path,
                        "file_size": file_size,
                        "modified_timestamp": modified,
                        "error": format!("Erro ao analisar JSON: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            // Retornar informações básicas do arquivo mesmo se não conseguir ler o conteúdo
            Ok(serde_json::json!({
                "file_name": file_name,
                "file_path": file_path,
                "file_size": file_size,
                "modified_timestamp": modified,
                "error": format!("Erro ao ler arquivo: {}", e)
            }))
        }
    }
}

// ==================== COMANDOS DE CONFIGURAÇÃO ====================

/// Carrega a configuração da aplicação
#[tauri::command]
pub async fn load_app_config() -> Result<AppConfig, TauriError> {
    config::load_config()
}

/// Salva a configuração da aplicação
#[tauri::command]
pub async fn save_app_config(config: AppConfig) -> Result<ConfigResult, TauriError> {
    match config::save_config(&config) {
        Ok(()) => Ok(ConfigResult {
            success: true,
            message: "Configuração salva com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Atualiza os diretórios de entrada e saída (versão melhorada)
#[tauri::command]
pub async fn update_config_directories(
    input_dir: Option<String>,
    output_dir: Option<String>
) -> Result<ConfigResult, TauriError> {
    match config::update_directories(input_dir, output_dir) {
        Ok(config) => Ok(ConfigResult {
            success: true,
            message: "Diretórios atualizados com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Adiciona um log ao histórico de processamento
#[tauri::command]
pub async fn add_config_log(
    message: String,
    log_type: String,
    session_id: Option<String>
) -> Result<ConfigResult, TauriError> {
    match config::add_processing_log(message, log_type, session_id) {
        Ok(config) => Ok(ConfigResult {
            success: true,
            message: "Log adicionado com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Limpa o histórico de logs
#[tauri::command]
pub async fn clear_config_logs() -> Result<ConfigResult, TauriError> {
    match config::clear_processing_logs() {
        Ok(config) => Ok(ConfigResult {
            success: true,
            message: "Logs limpos com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Atualiza configuração verbose
#[tauri::command]
pub async fn update_config_verbose(verbose: bool) -> Result<ConfigResult, TauriError> {
    match config::update_verbose_setting(verbose) {
        Ok(config) => Ok(ConfigResult {
            success: true,
            message: "Configuração verbose atualizada com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Obtém o diretório de configuração da aplicação
#[tauri::command]
pub async fn get_config_directory() -> Result<String, TauriError> {
    match config::get_config_dir() {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(e),
    }
}

/// Obtém o diretório da pasta PDF (Database/PDFs)
#[tauri::command]
pub async fn get_pdf_directory() -> Result<String, TauriError> {
    let current_exe = std::env::current_exe()
        .map_err(|e| TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao obter diretório do executável: {}", e),
            details: None,
        })?;
    
    let exe_dir = current_exe.parent()
        .ok_or_else(|| TauriError {
            error_type: "FileSystemError".to_string(),
            message: "Não foi possível obter o diretório pai do executável".to_string(),
            details: None,
        })?;
    
    let pdf_dir = exe_dir.join("Database").join("PDFs");
    
    // Criar a pasta se não existir
    if !pdf_dir.exists() {
        std::fs::create_dir_all(&pdf_dir)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar pasta Database/PDFs: {}", e),
                details: Some(pdf_dir.to_string_lossy().to_string()),
            })?;
    }
    
    Ok(pdf_dir.to_string_lossy().to_string())
}

/// Obtém o diretório da pasta de saída (Database/Resultados)
#[tauri::command]
pub async fn get_output_directory() -> Result<String, TauriError> {
    let current_exe = std::env::current_exe()
        .map_err(|e| TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao obter diretório do executável: {}", e),
            details: None,
        })?;
    
    let exe_dir = current_exe.parent()
        .ok_or_else(|| TauriError {
            error_type: "FileSystemError".to_string(),
            message: "Não foi possível obter o diretório pai do executável".to_string(),
            details: None,
        })?;
    
    let output_dir = exe_dir.join("Database").join("Resultados");
    
    // Criar a pasta se não existir
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar pasta Database/Resultados: {}", e),
                details: Some(output_dir.to_string_lossy().to_string()),
            })?;
    }
    
    Ok(output_dir.to_string_lossy().to_string())
}

/// Abre uma pasta no explorador de arquivos do sistema operacional
#[tauri::command]
pub async fn open_folder(path: String) -> Result<bool, TauriError> {
    let path_buf = PathBuf::from(&path);
    
    // Verificar se o caminho existe
    if !path_buf.exists() {
        return Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Caminho não encontrado: {}", path),
            details: Some(path.clone()),
        });
    }
    
    // Abrir pasta no sistema operacional
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| TauriError {
                error_type: "SystemError".to_string(),
                message: format!("Erro ao abrir pasta: {}", e),
                details: Some(path.clone()),
            })?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| TauriError {
                error_type: "SystemError".to_string(),
                message: format!("Erro ao abrir pasta: {}", e),
                details: Some(path.clone()),
            })?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| TauriError {
                error_type: "SystemError".to_string(),
                message: format!("Erro ao abrir pasta: {}", e),
                details: Some(path.clone()),
            })?;
    }
    
    Ok(true)
}

/// Processa múltiplos arquivos PDF na pasta PDF fixa
#[tauri::command]
pub async fn process_pdf_fixed_directory(
    verbose: bool,
    session_id: Option<String>,
    processing_state: State<'_, ProcessingState>
) -> Result<ProcessingResult, TauriError> {
    let session_id = session_id.unwrap_or_else(|| format!("pdf_fixed_{}", Utc::now().timestamp_millis()));
    
    // Obter pastas fixas
    let input_dir = get_pdf_directory().await?;
    let output_dir = get_output_directory().await?;
    
    let input_path = PathBuf::from(&input_dir);
    let output_path = PathBuf::from(&output_dir);
    
    // Contar arquivos PDF no diretório
    let total_files = WalkDir::new(&input_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
        .count();
    
    if total_files == 0 {
        return Err(TauriError {
            error_type: "ValidationError".to_string(),
            message: "Nenhum arquivo PDF encontrado na pasta PDF. Adicione arquivos PDF na pasta e tente novamente.".to_string(),
            details: Some(input_dir.clone()),
        });
    }
    
    // Inicializar estado de processamento
    {
        let mut state = processing_state.lock().unwrap();
        state.insert(session_id.clone(), ProcessingStatus {
            is_processing: true,
            current_file: None,
            processed_files: 0,
            total_files,
            errors: Vec::new(),
            progress_percentage: 0.0,
        });
    }
    
    // Processar todos os arquivos
    let processing_state_clone = processing_state.clone();
    let session_id_clone = session_id.clone();
    
    match pdf_processor::processar_diretorio_pdfs_com_progresso(
        &input_path, 
        &output_path, 
        verbose,
        |processed, total, current_file| {
            // Atualizar progresso em tempo real
            let mut state = processing_state_clone.lock().unwrap();
            if let Some(status) = state.get_mut(&session_id_clone) {
                status.processed_files = processed;
                status.total_files = total;
                status.current_file = current_file;
                status.progress_percentage = if total > 0 { (processed as f64 / total as f64) * 100.0 } else { 0.0 };
            }
        }
    ) {
        Ok(propostas) => {
            // Atualizar progresso final
            {
                let mut state = processing_state.lock().unwrap();
                if let Some(status) = state.get_mut(&session_id) {
                    status.processed_files = total_files;
                    status.progress_percentage = 100.0;
                    status.is_processing = false;
                }
            }
            
            // Salvar JSON consolidado
            if let Err(e) = pdf_processor::salvar_json_consolidado(&propostas, &output_path, "consolidado.json", verbose) {
                let error = TauriError {
                    error_type: "ProcessingError".to_string(),
                    message: format!("Erro ao salvar JSON consolidado: {}", e),
                    details: Some(input_dir.clone()),
                };
                
                {
                    let mut state = processing_state.lock().unwrap();
                    if let Some(status) = state.get_mut(&session_id) {
                        status.errors.push(error.message.clone());
                    }
                }
                
                return Err(error);
            }
            
            let json_file_path = output_path.join("resumo_geral.json");
            
            Ok(ProcessingResult {
                success: true,
                message: format!("Pasta PDF processada com sucesso: {} arquivos processados, {} propostas encontradas", total_files, propostas.len()),
                propostas,
                total_processed: total_files,
                json_file_path: Some(json_file_path.to_string_lossy().to_string()),
                session_id: Some(session_id),
            })
        }
        Err(e) => {
            let error = TauriError {
                error_type: "ProcessingError".to_string(),
                message: format!("Erro ao processar pasta PDF: {}", e),
                details: Some(input_dir.clone()),
            };
            
            // Atualizar estado com erro
            {
                let mut state = processing_state.lock().unwrap();
                if let Some(status) = state.get_mut(&session_id) {
                    status.is_processing = false;
                    status.errors.push(error.message.clone());
                }
            }
            
            Err(error)
        }
    }
}

/// Verifica se o diretório de output existe e cria um arquivo JSON de exemplo se necessário
#[tauri::command]
pub async fn verify_output_directory() -> Result<String, TauriError> {
    let output_dir = get_output_directory().await?;
    let output_path = PathBuf::from(&output_dir);
    
    // Contar arquivos JSON existentes
    let json_count = WalkDir::new(&output_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .count();
    
    if json_count == 0 {
        // Criar arquivo JSON de exemplo
        let exemplo_json = serde_json::json!({
            "data_geracao": "2024-01-01 12:00:00 UTC",
            "uasg": "000000",
            "pregao": "00000/2024",
            "processo": "00000000000000000",
            "total_propostas": 0,
            "valor_total": 0.0,
            "propostas": [],
            "exemplo": true,
            "mensagem": "Este é um arquivo de exemplo. Execute o processamento de PDFs para gerar dados reais."
        });
        
        let exemplo_path = output_path.join("exemplo.json");
        let json_content = serde_json::to_string_pretty(&exemplo_json)
            .map_err(|e| TauriError {
                error_type: "SerializationError".to_string(),
                message: format!("Erro ao serializar JSON de exemplo: {}", e),
                details: None,
            })?;
        
        std::fs::write(&exemplo_path, json_content)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar arquivo de exemplo: {}", e),
                details: Some(exemplo_path.to_string_lossy().to_string()),
            })?;
        
        Ok(format!("Diretório verificado. Criado arquivo de exemplo: {}", exemplo_path.to_string_lossy()))
    } else {
        Ok(format!("Diretório verificado. Encontrados {} arquivos JSON", json_count))
    }
}

// ==================== COMANDOS SICAF ====================

/// Obtém o diretório da pasta SICAF (Database/SICAF)
#[tauri::command]
pub async fn get_sicaf_directory() -> Result<String, TauriError> {
    let current_exe = std::env::current_exe()
        .map_err(|e| TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao obter diretório do executável: {}", e),
            details: None,
        })?;
    
    let exe_dir = current_exe.parent()
        .ok_or_else(|| TauriError {
            error_type: "FileSystemError".to_string(),
            message: "Não foi possível obter o diretório pai do executável".to_string(),
            details: None,
        })?;
    
    let sicaf_dir = exe_dir.join("Database").join("SICAF");
    
    // Criar a pasta se não existir
    if !sicaf_dir.exists() {
        std::fs::create_dir_all(&sicaf_dir)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar pasta Database/SICAF: {}", e),
                details: Some(sicaf_dir.to_string_lossy().to_string()),
            })?;
    }
    
    Ok(sicaf_dir.to_string_lossy().to_string())
}

/// Processa arquivos PDF SICAF na pasta SICAF fixa
#[tauri::command]
pub async fn process_sicaf_pdfs(verbose: bool) -> Result<ProcessingSicafResult, TauriError> {
    let sicaf_dir = get_sicaf_directory().await?;
    let sicaf_path = PathBuf::from(&sicaf_dir);
    
    match sicaf_processor::processar_sicaf_pdfs(&sicaf_path, verbose) {
        Ok(result) => {
            // Salvar dados em JSON se houver dados processados
            if !result.sicaf_data.is_empty() {
                let output_dir = get_output_directory().await?;
                let output_path = PathBuf::from(&output_dir);
                
                if let Err(e) = sicaf_processor::salvar_sicaf_json(&result.sicaf_data, &output_path, verbose) {
                    return Err(TauriError {
                        error_type: "ProcessingError".to_string(),
                        message: format!("Erro ao salvar dados SICAF: {}", e),
                        details: Some(sicaf_dir),
                    });
                }
            }
            
            Ok(result)
        }
        Err(e) => Err(TauriError {
            error_type: "ProcessingError".to_string(),
            message: format!("Erro ao processar PDFs SICAF: {}", e),
            details: Some(sicaf_dir),
        })
    }
}

/// Carrega dados SICAF do arquivo JSON
#[tauri::command]
pub async fn load_sicaf_data() -> Result<Vec<SicafData>, TauriError> {
    let output_dir = get_output_directory().await?;
    let sicaf_json_path = PathBuf::from(&output_dir).join("sicaf_dados.json");
    
    if !sicaf_json_path.exists() {
        return Ok(Vec::new());
    }
    
    match sicaf_processor::carregar_sicaf_json(&sicaf_json_path) {
        Ok(data) => Ok(data),
        Err(e) => Err(TauriError {
            error_type: "ProcessingError".to_string(),
            message: format!("Erro ao carregar dados SICAF: {}", e),
            details: Some(sicaf_json_path.to_string_lossy().to_string()),
        })
    }
}

/// Verifica se um CNPJ existe nos dados SICAF
#[tauri::command]
pub async fn verify_cnpj_sicaf(cnpj: String) -> Result<bool, TauriError> {
    let sicaf_data = load_sicaf_data().await?;
    Ok(sicaf_processor::verificar_cnpj_sicaf(&cnpj, &sicaf_data))
}

/// Obtém dados SICAF para um CNPJ específico
#[tauri::command]
pub async fn get_cnpj_sicaf_data(cnpj: String) -> Result<Option<SicafData>, TauriError> {
    let sicaf_data = load_sicaf_data().await?;
    match sicaf_processor::obter_dados_cnpj(&cnpj, &sicaf_data) {
        Some(data) => Ok(Some(data.clone())),
        None => Ok(None),
    }
}

/// Gera relatório de comparação entre licitação e SICAF
#[tauri::command]
pub async fn generate_sicaf_comparison_report(json_file_path: String) -> Result<String, TauriError> {
    // Carregar dados da licitação
    let licitacao_data = read_json_file(json_file_path.clone()).await?;
    
    let propostas: Vec<PropostaConsolidada> = if let Some(propostas_array) = licitacao_data.get("propostas").and_then(|p| p.as_array()) {
        propostas_array.iter().filter_map(|p| {
            serde_json::from_value(p.clone()).ok()
        }).collect()
    } else {
        return Err(TauriError {
            error_type: "ValidationError".to_string(),
            message: "Arquivo JSON não contém propostas válidas".to_string(),
            details: Some(json_file_path),
        });
    };
    
    // Carregar dados SICAF
    let sicaf_data = load_sicaf_data().await?;
    
    // Gerar relatório
    let output_dir = get_output_directory().await?;
    let output_path = PathBuf::from(&output_dir);
    
    match sicaf_processor::gerar_relatorio_comparacao(&propostas, &sicaf_data, &output_path, true) {
        Ok(()) => {
            let relatorio_path = output_path.join("relatorio_sicaf_comparacao.json");
            Ok(relatorio_path.to_string_lossy().to_string())
        }
        Err(e) => Err(TauriError {
            error_type: "ProcessingError".to_string(),
            message: format!("Erro ao gerar relatório de comparação: {}", e),
            details: Some(output_dir),
        })
    }
}

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

// ...existing code...
