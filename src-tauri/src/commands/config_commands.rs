use std::path::PathBuf;
use crate::types::*;
use crate::config;
use chrono::Utc;

/// Carrega a configuração da aplicação
#[tauri::command]
pub async fn load_app_config() -> Result<AppConfig, TauriError> {
    config::load_config()
}

/// Salva a configuração da aplicação
#[tauri::command]
pub async fn save_app_config(config: AppConfig) -> Result<ConfigResult, TauriError> {
    match config::save_config(&config) {
        Ok(_) => Ok(ConfigResult {
            success: true,
            message: "Configuração salva com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Atualiza os diretórios de entrada e saída
#[tauri::command]
pub async fn update_config_directories(
    input_dir: Option<String>,
    output_dir: Option<String>
) -> Result<ConfigResult, TauriError> {
    let mut config = config::load_config()?;
    
    if let Some(dir) = input_dir {
        config.last_input_directory = Some(dir);
    }
    
    if let Some(dir) = output_dir {
        config.last_output_directory = Some(dir);
    }
    
    config.updated_at = Utc::now().to_rfc3339();
    
    match config::save_config(&config) {
        Ok(_) => Ok(ConfigResult {
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
    let mut config = config::load_config()?;
    
    let log_entry = ProcessingLog {
        timestamp: Utc::now().to_rfc3339(),
        message,
        log_type,
        session_id,
    };
    
    config.processing_logs.push(log_entry);
    
    // Manter apenas os últimos logs
    if config.processing_logs.len() > config.max_logs {
        let total_logs = config.processing_logs.len();
        config.processing_logs = config.processing_logs
            .into_iter()
            .skip(total_logs - config.max_logs)
            .collect();
    }
    
    config.updated_at = Utc::now().to_rfc3339();
    
    match config::save_config(&config) {
        Ok(_) => Ok(ConfigResult {
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
    let mut config = config::load_config()?;
    
    config.processing_logs.clear();
    config.updated_at = Utc::now().to_rfc3339();
    
    match config::save_config(&config) {
        Ok(_) => Ok(ConfigResult {
            success: true,
            message: "Histórico de logs limpo com sucesso".to_string(),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Atualiza configuração verbose
#[tauri::command]
pub async fn update_config_verbose(verbose: bool) -> Result<ConfigResult, TauriError> {
    let mut config = config::load_config()?;
    
    config.verbose = verbose;
    config.updated_at = Utc::now().to_rfc3339();
    
    match config::save_config(&config) {
        Ok(_) => Ok(ConfigResult {
            success: true,
            message: format!("Configuração verbose atualizada para: {}", verbose),
            config: Some(config),
        }),
        Err(e) => Err(e),
    }
}

/// Debug e reparo do arquivo de configuração
#[tauri::command]
pub async fn debug_and_repair_config() -> Result<ConfigResult, TauriError> {
    use std::path::PathBuf;
    
    let mut debug_info = String::new();
    debug_info.push_str("=== DEBUG E REPARO DA CONFIGURAÇÃO ===\n\n");
    
    // Obter diretório de configuração
    let config_dir = match config::get_config_dir() {
        Ok(dir) => {
            debug_info.push_str(&format!("✅ Diretório de configuração: {}\n", dir.display()));
            PathBuf::from(dir)
        }
        Err(e) => {
            debug_info.push_str(&format!("❌ Erro ao obter diretório de configuração: {:?}\n", e));
            return Err(e);
        }
    };
    
    let config_path = config_dir.join("licitacao360_config.json");
    debug_info.push_str(&format!("📁 Caminho do arquivo de configuração: {}\n", config_path.display()));
    
    // Verificar se o arquivo existe
    if config_path.exists() {
        debug_info.push_str("✅ Arquivo de configuração existe\n");
        
        // Tentar ler o arquivo
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                debug_info.push_str(&format!("✅ Arquivo lido com sucesso ({} bytes)\n", content.len()));
                
                // Tentar fazer parse do JSON
                match serde_json::from_str::<AppConfig>(&content) {
                    Ok(_) => {
                        debug_info.push_str("✅ JSON válido e configuração carregada com sucesso\n");
                    }
                    Err(e) => {
                        debug_info.push_str(&format!("❌ Erro ao fazer parse do JSON: {}\n", e));
                        debug_info.push_str("🔧 Criando nova configuração...\n");
                        create_new_config_with_backup(&config_path, &mut debug_info);
                    }
                }
            }
            Err(e) => {
                debug_info.push_str(&format!("❌ Erro ao ler arquivo: {}\n", e));
                debug_info.push_str("🔧 Criando nova configuração...\n");
                create_new_config_with_backup(&config_path, &mut debug_info);
            }
        }
    } else {
        debug_info.push_str("⚠️ Arquivo de configuração não existe\n");
        debug_info.push_str("🔧 Criando nova configuração...\n");
        
        // Criar diretório se não existir
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            debug_info.push_str(&format!("❌ Erro ao criar diretório: {}\n", e));
            return Err(TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar diretório de configuração: {}", e),
                details: Some(config_dir.to_string_lossy().to_string()),
            });
        }
        
        create_new_config_with_backup(&config_path, &mut debug_info);
    }
    
    debug_info.push_str("\n=== REPARO CONCLUÍDO ===\n");
    
    Ok(ConfigResult {
        success: true,
        message: debug_info,
        config: config::load_config().ok(),
    })
}

/// Inicializa a aplicação criando diretórios padrão e configuração
#[tauri::command]
pub async fn initialize_application() -> Result<ConfigResult, TauriError> {
    use crate::commands::directory_commands::{get_config_directory, get_pdf_directory, get_output_directory};
    
    // Garantir que os diretórios existem
    let _config_dir = get_config_directory().await?;
    let _pdf_dir = get_pdf_directory().await?;
    let _output_dir = get_output_directory().await?;
    
    // Carregar ou criar configuração
    let config = match config::load_config() {
        Ok(config) => config,
        Err(_) => {
            // Criar configuração padrão se não existir
            let default_config = config::create_default_config();
            config::save_config(&default_config)?;
            default_config
        }
    };
    
    Ok(ConfigResult {
        success: true,
        message: "Aplicação inicializada com sucesso".to_string(),
        config: Some(config),
    })
}

/// Obtém informações detalhadas dos diretórios da aplicação
#[tauri::command]
pub async fn get_app_directories_info() -> Result<serde_json::Value, TauriError> {
    use crate::commands::directory_commands::{get_config_directory, get_pdf_directory, get_output_directory};
    
    let home_dir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "N/A".to_string());
    
    let config_dir = get_config_directory().await?;
    let pdf_dir = get_pdf_directory().await?;
    let output_dir = get_output_directory().await?;
    
    // Verificar se os diretórios e arquivos existem
    let config_file_path = std::path::PathBuf::from(&config_dir).join("licitacao360_config.json");
    let config_file_exists = config_file_path.exists();
    let pdf_directory_exists = std::path::PathBuf::from(&pdf_dir).exists();
    let output_directory_exists = std::path::PathBuf::from(&output_dir).exists();
    
    Ok(serde_json::json!({
        "home_directory": home_dir,
        "config_directory": config_dir,
        "default_pdf_directory": pdf_dir,
        "default_output_directory": output_dir,
        "config_file_path": config_file_path.to_string_lossy(),
        "config_file_exists": config_file_exists,
        "pdf_directory_exists": pdf_directory_exists,
        "output_directory_exists": output_directory_exists
    }))
}

/// Obtém o diretório PDF padrão
#[tauri::command]
pub async fn get_default_pdf_directory() -> Result<String, TauriError> {
    use crate::commands::directory_commands::get_pdf_directory;
    get_pdf_directory().await
}

/// Obtém o diretório de saída padrão
#[tauri::command]
pub async fn get_default_output_directory() -> Result<String, TauriError> {
    use crate::commands::directory_commands::get_output_directory;
    get_output_directory().await
}

/// Garante que um diretório existe
#[tauri::command]
pub async fn ensure_directory_exists(path: String) -> Result<bool, TauriError> {
    let path_buf = std::path::PathBuf::from(&path);
    
    if path_buf.exists() {
        return Ok(true);
    }
    
    match std::fs::create_dir_all(&path_buf) {
        Ok(_) => Ok(true),
        Err(e) => Err(TauriError {
            error_type: "FileSystemError".to_string(),
            message: format!("Erro ao criar diretório: {}", e),
            details: Some(path),
        })
    }
}

/// Obtém o diretório home do usuário
#[tauri::command]
pub async fn get_user_home_directory() -> Result<String, TauriError> {
    match dirs::home_dir() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err(TauriError {
            error_type: "SystemError".to_string(),
            message: "Não foi possível obter o diretório home do usuário".to_string(),
            details: None,
        })
    }
}

/// Atualiza o diretório PDF na configuração
#[tauri::command]
pub async fn update_pdf_directory(path: String) -> Result<ConfigResult, TauriError> {
    update_config_directories(Some(path), None).await
}

/// Atualiza o diretório de saída na configuração
#[tauri::command]
pub async fn update_output_directory(path: String) -> Result<ConfigResult, TauriError> {
    update_config_directories(None, Some(path)).await
}

fn create_new_config_with_backup(config_path: &PathBuf, debug_info: &mut String) {
    // Fazer backup do arquivo corrompido se existir
    if config_path.exists() {
        let backup_path = config_path.with_extension("json.backup");
        if let Err(e) = std::fs::copy(config_path, &backup_path) {
            debug_info.push_str(&format!("⚠️ Erro ao criar backup: {}\n", e));
        } else {
            debug_info.push_str(&format!("💾 Backup criado em: {}\n", backup_path.display()));
        }
    }
    
    // Criar nova configuração
    let new_config = AppConfig {
        last_input_directory: None,
        last_output_directory: None,
        verbose: false,
        processing_logs: Vec::new(),
        max_logs: 1000,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };
    
    match serde_json::to_string_pretty(&new_config) {
        Ok(json_content) => {
            match std::fs::write(config_path, json_content) {
                Ok(_) => {
                    debug_info.push_str("✅ Nova configuração criada com sucesso\n");
                }
                Err(e) => {
                    debug_info.push_str(&format!("❌ Erro ao escrever nova configuração: {}\n", e));
                }
            }
        }
        Err(e) => {
            debug_info.push_str(&format!("❌ Erro ao serializar nova configuração: {}\n", e));
        }
    }
}
