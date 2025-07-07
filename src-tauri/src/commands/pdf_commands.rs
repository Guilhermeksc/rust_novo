use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::State;
use crate::types::*;
use crate::pdf_processor;
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
                return Err(TauriError {
                    error_type: "ProcessingError".to_string(),
                    message: format!("Erro ao salvar JSON consolidado: {}", e),
                    details: Some(output_dir),
                });
            }
            
            let json_file_path = output_path.join("resumo_geral.json");
            
            Ok(ProcessingResult {
                success: true,
                message: format!("Processamento concluído: {} arquivos processados", total_files),
                propostas,
                total_processed: total_files,
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
                    status.errors.push(format!("Erro ao processar diretório: {}", e));
                }
            }
            
            Err(TauriError {
                error_type: "ProcessingError".to_string(),
                message: format!("Erro ao processar diretório: {}", e),
                details: Some(input_dir),
            })
        }
    }
}

/// Processa múltiplos arquivos PDF na pasta PDF fixa
#[tauri::command]
pub async fn process_pdf_fixed_directory(
    verbose: bool,
    session_id: Option<String>,
    processing_state: State<'_, ProcessingState>
) -> Result<ProcessingResult, TauriError> {
    let input_dir = super::directory_commands::get_pdf_directory().await?;
    let output_dir = super::directory_commands::get_output_directory().await?;
    
    process_pdf_directory(input_dir, output_dir, verbose, session_id, processing_state).await
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
            error_type: "NotFound".to_string(),
            message: format!("Sessão de processamento não encontrada: {}", session_id),
            details: Some(session_id),
        })
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
        return Ok(false);
    }
    
    // Verificar se é um arquivo PDF
    if path.extension().map_or(false, |ext| ext == "pdf") {
        Ok(true)
    } else {
        Ok(false)
    }
}
