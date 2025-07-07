use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use pdf_extract::extract_text;
use crate::types::{SicafData, ProcessingSicafResult, PropostaConsolidada};

/// Processa todos os arquivos PDF SICAF de um diretório
pub fn processar_sicaf_pdfs(sicaf_dir: &Path, verbose: bool) -> Result<ProcessingSicafResult> {
    if !sicaf_dir.exists() {
        return Err(anyhow::anyhow!("Diretório SICAF não encontrado: {}", sicaf_dir.display()));
    }

    let mut sicaf_data_list: Vec<SicafData> = Vec::new();
    let mut processed_count = 0;

    // Coletar todos os arquivos PDF
    let pdf_files: Vec<_> = WalkDir::new(sicaf_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
        .collect();

    if pdf_files.is_empty() {
        return Ok(ProcessingSicafResult {
            success: true,
            message: "Nenhum arquivo PDF encontrado na pasta SICAF".to_string(),
            processed_count: 0,
            sicaf_data: Vec::new(),
            session_id: None,
        });
    }

    for entry in pdf_files {
        if verbose {
            println!("Processando arquivo SICAF: {:?}", entry.path());
        }

        match processar_pdf_sicaf(entry.path(), verbose) {
            Ok(Some(sicaf_data)) => {
                sicaf_data_list.push(sicaf_data);
                processed_count += 1;
                if verbose {
                    println!("✓ Arquivo processado com sucesso: {:?}", entry.path());
                }
            }
            Ok(None) => {
                if verbose {
                    println!("⚠ Dados SICAF não encontrados no arquivo: {:?}", entry.path());
                }
            }
            Err(e) => {
                eprintln!("✗ Erro ao processar {:?}: {}", entry.path(), e);
            }
        }
    }

    Ok(ProcessingSicafResult {
        success: true,
        message: format!("Processamento concluído: {} arquivos processados", processed_count),
        processed_count,
        sicaf_data: sicaf_data_list,
        session_id: Some(format!("sicaf_{}", Utc::now().timestamp_millis())),
    })
}

/// Processa um único arquivo PDF SICAF
fn processar_pdf_sicaf(pdf_path: &Path, verbose: bool) -> Result<Option<SicafData>> {
    // Extrair texto do PDF
    let text = extract_text(pdf_path)?;
    
    if verbose {
        println!("📝 Texto extraído do SICAF: {} caracteres", text.len());
    }

    // Extrair dados principais do SICAF
    let mut sicaf_data = match extrair_dados_sicaf(&text) {
        Some(data) => data,
        None => return Ok(None),
    };

    // Extrair dados do responsável legal
    if let Some(responsavel_data) = extrair_dados_responsavel(&text) {
        sicaf_data.cpf_responsavel = Some(responsavel_data.cpf);
        sicaf_data.nome_responsavel = Some(responsavel_data.nome);
    }

    if verbose {
        println!("✅ Dados SICAF extraídos - CNPJ: {}, Empresa: {}", sicaf_data.cnpj, sicaf_data.empresa);
    }

    Ok(Some(sicaf_data))
}

/// Extrai dados principais do SICAF usando regex
fn extrair_dados_sicaf(texto: &str) -> Option<SicafData> {
    // Padrão regex baseado no exemplo Python
    let dados_sicaf_pattern = r"(?s)CNPJ:\s*(?P<cnpj>[\d./-]+)\s*(?:DUNS®:\s*(?P<duns>[\d]+)\s*)?Razão Social:\s*(?P<empresa>.*?)\s*Nome Fantasia:\s*(?P<nome_fantasia>.*?)\s*Situação do Fornecedor:\s*(?P<situacao_cadastro>.*?)\s*Data de Vencimento do Cadastro:\s*(?P<data_vencimento>\d{2}/\d{2}/\d{4})\s*Dados do Nível.*?Dados para Contato\s*CEP:\s*(?P<cep>[\d.-]+)\s*Endereço:\s*(?P<endereco>.*?)\s*Município\s*/\s*UF:\s*(?P<municipio>.*?)\s*/\s*(?P<uf>.*?)\s*Telefone:\s*(?P<telefone>.*?)\s*E-mail:\s*(?P<email>.*?)\s*Dados do Responsável Legal";

    let re = Regex::new(dados_sicaf_pattern).ok()?;
    
    if let Some(caps) = re.captures(texto) {
        Some(SicafData {
            cnpj: caps.name("cnpj")?.as_str().trim().to_string(),
            duns: caps.name("duns").map(|m| m.as_str().trim().to_string()),
            empresa: caps.name("empresa")?.as_str().trim().to_string(),
            nome_fantasia: caps.name("nome_fantasia")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            situacao_cadastro: caps.name("situacao_cadastro")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            data_vencimento: caps.name("data_vencimento")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            cep: caps.name("cep")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            endereco: caps.name("endereco")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            municipio: caps.name("municipio")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            uf: caps.name("uf")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            telefone: caps.name("telefone")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            email: caps.name("email")
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty()),
            cpf_responsavel: None,
            nome_responsavel: None,
        })
    } else {
        None
    }
}

/// Dados do responsável legal
struct ResponsavelData {
    cpf: String,
    nome: String,
}

/// Extrai dados do responsável legal usando regex
fn extrair_dados_responsavel(texto: &str) -> Option<ResponsavelData> {
    let dados_responsavel_pattern = r"(?s)Dados do Responsável Legal\s*CPF:\s*(?P<cpf>\d{3}\.\d{3}\.\d{3}-\d{2})\s*Nome:\s*(?P<nome>[^\n\r]*?)(?:\s*Dados do Responsável pelo Cadastro|\s*Emitido em:|\s*CPF:|$)";

    let re = Regex::new(dados_responsavel_pattern).ok()?;
    
    if let Some(caps) = re.captures(texto) {
        Some(ResponsavelData {
            cpf: caps.name("cpf")?.as_str().trim().to_string(),
            nome: caps.name("nome")?.as_str().trim().to_string(),
        })
    } else {
        None
    }
}

/// Salva dados SICAF em arquivo JSON
pub fn salvar_sicaf_json(sicaf_data: &[SicafData], output_dir: &Path, verbose: bool) -> Result<()> {
    let data_geracao = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    let sicaf_json = serde_json::json!({
        "data_geracao": data_geracao,
        "total_registros": sicaf_data.len(),
        "registros_sicaf": sicaf_data
    });

    let json_path = output_dir.join("sicaf_dados.json");
    let json_content = serde_json::to_string_pretty(&sicaf_json)
        .context("Erro ao serializar dados SICAF")?;

    fs::write(&json_path, json_content)
        .context("Erro ao salvar arquivo JSON SICAF")?;

    if verbose {
        println!("📄 Dados SICAF salvos em: {:?}", json_path);
    }

    Ok(())
}

/// Carrega dados SICAF de um arquivo JSON
pub fn carregar_sicaf_json(json_path: &Path) -> Result<Vec<SicafData>> {
    let json_content = fs::read_to_string(json_path)
        .context("Erro ao ler arquivo JSON SICAF")?;

    let json_data: serde_json::Value = serde_json::from_str(&json_content)
        .context("Erro ao parsear JSON SICAF")?;

    let registros = json_data["registros_sicaf"]
        .as_array()
        .context("Campo 'registros_sicaf' não encontrado no JSON")?;

    let mut sicaf_data = Vec::new();
    for registro in registros {
        let data: SicafData = serde_json::from_value(registro.clone())
            .context("Erro ao deserializar registro SICAF")?;
        sicaf_data.push(data);
    }

    Ok(sicaf_data)
}

/// Verifica se um CNPJ existe nos dados SICAF
pub fn verificar_cnpj_sicaf(cnpj: &str, sicaf_data: &[SicafData]) -> bool {
    // Normalizar CNPJ removendo formatação
    let cnpj_normalizado = cnpj.replace(".", "").replace("/", "").replace("-", "");
    
    sicaf_data.iter().any(|data| {
        let cnpj_data_normalizado = data.cnpj.replace(".", "").replace("/", "").replace("-", "");
        cnpj_data_normalizado == cnpj_normalizado
    })
}

/// Obtém dados SICAF para um CNPJ específico
pub fn obter_dados_cnpj<'a>(cnpj: &str, sicaf_data: &'a [SicafData]) -> Option<&'a SicafData> {
    // Normalizar CNPJ removendo formatação
    let cnpj_normalizado = cnpj.replace(".", "").replace("/", "").replace("-", "");
    
    sicaf_data.iter().find(|data| {
        let cnpj_data_normalizado = data.cnpj.replace(".", "").replace("/", "").replace("-", "");
        cnpj_data_normalizado == cnpj_normalizado
    })
}

/// Gera relatório de comparação entre licitação e SICAF
pub fn gerar_relatorio_comparacao(
    propostas: &[PropostaConsolidada],
    sicaf_data: &[SicafData],
    output_dir: &Path,
    verbose: bool,
) -> Result<()> {
    let mut relatorio = Vec::new();
    
    for proposta in propostas {
        let sicaf_encontrado = obter_dados_cnpj(&proposta.cnpj, sicaf_data);
        
        let status = if sicaf_encontrado.is_some() {
            "SICAF Encontrado"
        } else {
            "SICAF Não Encontrado"
        };
        
        relatorio.push(serde_json::json!({
            "cnpj": proposta.cnpj,
            "fornecedor": proposta.fornecedor,
            "status_sicaf": status,
            "dados_sicaf": sicaf_encontrado,
            "proposta": {
                "item": proposta.item,
                "valor_adjudicado": proposta.valor_adjudicado,
                "uasg": proposta.uasg,
                "pregao": proposta.pregao
            }
        }));
    }
    
    let data_geracao = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let relatorio_final = serde_json::json!({
        "data_geracao": data_geracao,
        "total_propostas": propostas.len(),
        "sicaf_encontrados": relatorio.iter().filter(|r| r["status_sicaf"] == "SICAF Encontrado").count(),
        "sicaf_nao_encontrados": relatorio.iter().filter(|r| r["status_sicaf"] == "SICAF Não Encontrado").count(),
        "relatorio": relatorio
    });

    let relatorio_path = output_dir.join("relatorio_sicaf_comparacao.json");
    let relatorio_content = serde_json::to_string_pretty(&relatorio_final)
        .context("Erro ao serializar relatório de comparação")?;

    fs::write(&relatorio_path, relatorio_content)
        .context("Erro ao salvar relatório de comparação")?;

    if verbose {
        println!("📊 Relatório de comparação salvo em: {:?}", relatorio_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrair_dados_sicaf() {
        let texto_exemplo = r#"
            CNPJ: 12.345.678/0001-90
            DUNS®: 123456789
            Razão Social: EMPRESA TESTE LTDA
            Nome Fantasia: TESTE LTDA
            Situação do Fornecedor: HABILITADO
            Data de Vencimento do Cadastro: 31/12/2024
            Dados do Nível 1 - Credenciamento
            Dados para Contato
            CEP: 01234-567
            Endereço: RUA TESTE, 123 - CENTRO
            Município / UF: SÃO PAULO / SP
            Telefone: (11) 1234-5678
            E-mail: teste@empresa.com.br
            Dados do Responsável Legal
        "#;

        let resultado = extrair_dados_sicaf(texto_exemplo);
        assert!(resultado.is_some());
        
        let dados = resultado.unwrap();
        assert_eq!(dados.cnpj, "12.345.678/0001-90");
        assert_eq!(dados.empresa, "EMPRESA TESTE LTDA");
        assert_eq!(dados.nome_fantasia, Some("TESTE LTDA".to_string()));
        assert_eq!(dados.situacao_cadastro, Some("HABILITADO".to_string()));
        assert_eq!(dados.data_vencimento, Some("31/12/2024".to_string()));
        assert_eq!(dados.cep, Some("01234-567".to_string()));
        assert_eq!(dados.endereco, Some("RUA TESTE, 123 - CENTRO".to_string()));
        assert_eq!(dados.municipio, Some("SÃO PAULO".to_string()));
        assert_eq!(dados.uf, Some("SP".to_string()));
        assert_eq!(dados.telefone, Some("(11) 1234-5678".to_string()));
        assert_eq!(dados.email, Some("teste@empresa.com.br".to_string()));
    }

    #[test]
    fn test_extrair_dados_responsavel() {
        let texto_exemplo = r#"
            Dados do Responsável Legal
            CPF: 123.456.789-00
            Nome: JOÃO DA SILVA
            Dados do Responsável pelo Cadastro
        "#;

        let resultado = extrair_dados_responsavel(texto_exemplo);
        assert!(resultado.is_some());
        
        let dados = resultado.unwrap();
        assert_eq!(dados.cpf, "123.456.789-00");
        assert_eq!(dados.nome, "JOÃO DA SILVA");
    }

    #[test]
    fn test_verificar_cnpj_sicaf() {
        let sicaf_data = vec![
            SicafData {
                cnpj: "12.345.678/0001-90".to_string(),
                duns: None,
                empresa: "TESTE LTDA".to_string(),
                nome_fantasia: None,
                situacao_cadastro: None,
                data_vencimento: None,
                cep: None,
                endereco: None,
                municipio: None,
                uf: None,
                telefone: None,
                email: None,
                cpf_responsavel: None,
                nome_responsavel: None,
            }
        ];

        // Deve encontrar com formatação
        assert!(verificar_cnpj_sicaf("12.345.678/0001-90", &sicaf_data));
        
        // Deve encontrar sem formatação
        assert!(verificar_cnpj_sicaf("12345678000190", &sicaf_data));
        
        // Não deve encontrar CNPJ inexistente
        assert!(!verificar_cnpj_sicaf("98.765.432/0001-10", &sicaf_data));
    }
} 