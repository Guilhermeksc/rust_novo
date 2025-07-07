use std::path::PathBuf;
use crate::types::{TauriError, ProcessingSicafResult, SicafData, PropostaConsolidada};
use crate::sicaf_processor;
use crate::commands::directory_commands::{get_sicaf_directory, get_output_directory};
use crate::commands::json_commands::read_json_file;

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
