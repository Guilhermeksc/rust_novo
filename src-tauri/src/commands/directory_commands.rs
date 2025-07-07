use std::path::PathBuf;
use crate::types::TauriError;

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

/// Obtém o diretório da pasta de configuração
#[tauri::command]
pub async fn get_config_directory() -> Result<String, TauriError> {
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
    
    let config_dir = exe_dir.join("Database").join("Config");
    
    // Criar a pasta se não existir
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar pasta Database/Config: {}", e),
                details: Some(config_dir.to_string_lossy().to_string()),
            })?;
    }
    
    Ok(config_dir.to_string_lossy().to_string())
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

/// Verifica e cria o diretório de saída, retornando informações sobre ele
#[tauri::command]
pub async fn verify_output_directory() -> Result<String, TauriError> {
    let output_dir = get_output_directory().await?;
    let output_path = PathBuf::from(&output_dir);
    
    // Verificar se existem arquivos JSON no diretório
    let json_count = walkdir::WalkDir::new(&output_path)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .count();
    
    // Criar um arquivo de exemplo se não houver arquivos JSON
    if json_count == 0 {
        let exemplo_path = output_path.join("exemplo_resultado.json");
        let exemplo_content = serde_json::json!({
            "info": "Esta pasta contém os resultados do processamento de PDFs",
            "formato": "Os arquivos JSON gerados contêm as propostas extraídas dos PDFs",
            "exemplo_proposta": {
                "pregao": "787000-90008/2024",
                "processo": "62055002454202331",
                "uasg": "787000",
                "fornecedor": "EMPRESA EXEMPLO LTDA",
                "cnpj": "00.000.000/0001-00",
                "item": "1",
                "descricao": "Exemplo de descrição do item",
                "quantidade": "1",
                "valor_estimado": "R$ 1.000,00",
                "valor_adjudicado": "R$ 950,00",
                "marca_fabricante": "MARCA EXEMPLO",
                "modelo_versao": "MODELO V1.0"
            }
        });
        
        std::fs::write(&exemplo_path, serde_json::to_string_pretty(&exemplo_content).unwrap())
            .map_err(|e| TauriError {
                error_type: "FileSystemError".to_string(),
                message: format!("Erro ao criar arquivo de exemplo: {}", e),
                details: Some(exemplo_path.to_string_lossy().to_string()),
            })?;
    }
    
    Ok(format!("Pasta de resultados verificada: {} ({} arquivos JSON encontrados)", 
        output_dir, json_count))
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
