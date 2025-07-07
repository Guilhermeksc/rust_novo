use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingArgs {
    pub input_dir: String,
    pub output_dir: String,
    pub file: Option<String>,
    pub verbose: bool,
    pub json_output: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingStatus {
    pub is_processing: bool,
    pub current_file: Option<String>,
    pub processed_files: usize,
    pub total_files: usize,
    pub errors: Vec<String>,
    pub progress_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropostaAdjudicada {
    pub item: String,
    pub grupo: Option<String>,
    pub descricao: String,
    pub quantidade: String,
    pub valor_estimado: String,
    pub valor_adjudicado: String,
    pub fornecedor: String,
    pub cnpj: String,
    pub melhor_lance: String,
    pub responsavel: String,
    pub cpf_responsavel: String,
    pub marca_fabricante: String,
    pub modelo_versao: String,
    pub tipo_formato: String, // "individual" ou "grupo"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PropostaConsolidada {
    pub uasg: String,
    pub pregao: String,
    pub processo: String,
    pub item: String,
    pub grupo: Option<String>,
    pub quantidade: String,
    pub descricao: String,
    pub valor_estimado: String,
    pub valor_adjudicado: String,
    pub fornecedor: String,
    pub cnpj: String,
    pub marca_fabricante: String,
    pub modelo_versao: String,
    pub responsavel: String,
    pub melhor_lance: String,
    pub tipo_formato: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicitacaoConsolidada {
    pub uasg: String,
    pub pregao: String,
    pub processo: String,
    pub total_propostas: usize,
    pub valor_total: f64,
    pub propostas: Vec<PropostaConsolidada>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsolidadoJson {
    pub data_geracao: String,
    pub total_licitacoes: usize,
    pub total_propostas: usize,
    pub valor_total_geral: f64,
    pub licitacoes: HashMap<String, LicitacaoConsolidada>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatorioLicitacao {
    pub uasg: String,
    pub pregao: String,
    pub processo: String,
    pub data_homologacao: String,
    pub responsavel: String,
    pub valor_total: f64,
    pub propostas: Vec<PropostaAdjudicada>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanceItem {
    pub data_hora: String,
    pub participante: String,
    pub valor: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingResult {
    pub success: bool,
    pub message: String,
    pub propostas: Vec<PropostaConsolidada>,
    pub total_processed: usize,
    pub json_file_path: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TauriError {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingLog {
    pub timestamp: String,
    pub message: String,
    pub log_type: String, // 'info', 'success', 'error', 'progress'
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub last_input_directory: Option<String>,
    pub last_output_directory: Option<String>,
    pub verbose: bool,
    pub processing_logs: Vec<ProcessingLog>,
    pub max_logs: usize,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigResult {
    pub success: bool,
    pub message: String,
    pub config: Option<AppConfig>,
}

/// Estrutura para dados do SICAF
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SicafData {
    pub cnpj: String,
    pub duns: Option<String>,
    pub empresa: String,
    pub nome_fantasia: Option<String>,
    pub situacao_cadastro: Option<String>,
    pub data_vencimento: Option<String>,
    pub cep: Option<String>,
    pub endereco: Option<String>,
    pub municipio: Option<String>,
    pub uf: Option<String>,
    pub telefone: Option<String>,
    pub email: Option<String>,
    pub cpf_responsavel: Option<String>,
    pub nome_responsavel: Option<String>,
}

/// Estrutura para resultado do processamento SICAF
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingSicafResult {
    pub success: bool,
    pub message: String,
    pub processed_count: usize,
    pub sicaf_data: Vec<SicafData>,
    pub session_id: Option<String>,
}