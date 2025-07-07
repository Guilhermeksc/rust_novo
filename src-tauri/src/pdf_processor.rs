use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use pdf_extract::extract_text;
use std::collections::{HashSet, HashMap};
use crate::types::*;

/// Processa um arquivo PDF específico e retorna as propostas consolidadas
pub fn processar_pdf_com_consolidacao(pdf_path: &Path, output_dir: &Path, verbose: bool) -> Result<Vec<PropostaConsolidada>> {
    if verbose {
        println!("📄 Processando: {}", pdf_path.display());
    }
    
    // Extrair texto do PDF
    let text = extract_text(pdf_path)?;
    
    if verbose {
        println!("📝 Texto extraído: {} caracteres", text.len());
    }
    
    // Extrair informações gerais
    let mut relatorio = RelatorioLicitacao {
        uasg: extrair_uasg(&text),
        pregao: extrair_pregao(&text),
        processo: extrair_processo(&text),
        data_homologacao: extrair_data_homologacao(&text),
        responsavel: extrair_responsavel(&text),
        valor_total: 0.0,
        propostas: Vec::new(),
    };
    
    // Tentar extrair propostas no formato de grupo primeiro
    let mut propostas_grupo = extrair_propostas_grupo(&text, verbose);
    
    // Se não encontrou propostas de grupo, tentar formato individual
    if propostas_grupo.is_empty() {
        let mut propostas_individuais = extrair_propostas_individuais(&text, verbose);
        relatorio.propostas.append(&mut propostas_individuais);
        
        if verbose {
            println!("📊 Formato individual detectado: {} propostas encontradas", relatorio.propostas.len());
        }
    } else {
        relatorio.propostas.append(&mut propostas_grupo);
        
        if verbose {
            println!("📊 Formato de grupo detectado: {} propostas encontradas", relatorio.propostas.len());
        }
    }
    
    // Calcular valor total
    relatorio.valor_total = relatorio.propostas.iter()
        .map(|p| converter_valor_para_float(&p.valor_adjudicado))
        .sum();
    
    if verbose {
        println!("💰 Valor total calculado: R$ {:.2}", relatorio.valor_total);
    }
    
    // Gerar nome do arquivo de saída
    let nome_arquivo = pdf_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    
    let output_path = output_dir.join(format!("{}.md", nome_arquivo));
    
    // Gerar Markdown estruturado
    let markdown = gerar_markdown(&relatorio)?;
    
    // Salvar arquivo
    fs::write(&output_path, markdown)
        .context("Erro ao salvar arquivo Markdown")?;
    
    if verbose {
        println!("Arquivo salvo em: {:?}", output_path);
    }
    
    // Converter propostas para formato consolidado
    let propostas_consolidadas: Vec<PropostaConsolidada> = relatorio.propostas.iter().map(|p| {
        PropostaConsolidada {
            uasg: relatorio.uasg.clone(),
            pregao: relatorio.pregao.clone(),
            processo: relatorio.processo.clone(),
            item: p.item.clone(),
            grupo: p.grupo.clone(),
            quantidade: p.quantidade.clone(),
            descricao: p.descricao.clone(),
            valor_estimado: p.valor_estimado.clone(),
            valor_adjudicado: p.valor_adjudicado.clone(),
            fornecedor: p.fornecedor.clone(),
            cnpj: p.cnpj.clone(),
            marca_fabricante: p.marca_fabricante.clone(),
            modelo_versao: p.modelo_versao.clone(),
            responsavel: p.responsavel.clone(),
            melhor_lance: p.melhor_lance.clone(),
            tipo_formato: p.tipo_formato.clone(),
        }
    }).collect();
    
    Ok(propostas_consolidadas)
}

/// Processa todos os arquivos PDF de um diretório
pub fn processar_diretorio_pdfs_com_progresso<F>(
    input_dir: &Path, 
    output_dir: &Path, 
    verbose: bool,
    mut progress_callback: F
) -> Result<Vec<PropostaConsolidada>> 
where
    F: FnMut(usize, usize, Option<String>),
{
    let mut todas_propostas: Vec<PropostaConsolidada> = Vec::new();
    
    // Criar diretório de saída se não existir
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .context("Erro ao criar diretório de saída")?;
    }
    
    // Coletar todos os arquivos PDF primeiro
    let pdf_files: Vec<_> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "pdf"))
        .collect();
    
    let total_files = pdf_files.len();
    
    // Processar cada arquivo
    for (index, entry) in pdf_files.iter().enumerate() {
        let current_file = entry.path().to_string_lossy().to_string();
        
        // Atualizar progresso antes de processar o arquivo
        progress_callback(index, total_files, Some(current_file.clone()));
        
        if verbose {
            println!("Processando: {:?}", entry.path());
        }
        
        match processar_pdf_com_consolidacao(entry.path(), output_dir, verbose) {
            Ok(propostas) => {
                todas_propostas.extend(propostas);
                if verbose {
                    println!("✓ Processado com sucesso: {:?}", entry.path());
                }
            }
            Err(e) => {
                eprintln!("✗ Erro ao processar {:?}: {}", entry.path(), e);
            }
        }
        
        // Atualizar progresso após processar o arquivo
        progress_callback(index + 1, total_files, None);
    }
    
    Ok(todas_propostas)
}

/// Processa todos os arquivos PDF de um diretório (versão original mantida para compatibilidade)
pub fn processar_diretorio_pdfs(input_dir: &Path, output_dir: &Path, verbose: bool) -> Result<Vec<PropostaConsolidada>> {
    processar_diretorio_pdfs_com_progresso(input_dir, output_dir, verbose, |_, _, _| {})
}

/// Extrai propostas no formato individual
fn extrair_propostas_individuais(text: &str, verbose: bool) -> Vec<PropostaAdjudicada> {
    let mut propostas = Vec::new();
    let mut cnpjs_processados = HashSet::new();

    // Padrões para formato individual
    let re_adjucado_negociado = Regex::new(
        r"Adjucado e Homologado por CPF\s*(?P<cpf>[\d\.\-\*]+)\s*-\s*(?P<responsavel>[^,]+),?\s*para\s+(?P<fornecedor>[^,]+),\s*CNPJ\s*(?P<cnpj>[\d\.\-/]+),\s*melhor\s+lance:\s*R\$\s*(?P<melhor_lance>[\d,\.]+).*?valor\s+negociado:\s*R\$\s*(?P<valor_negociado>[\d,\.]+)"
    ).unwrap();

    let re_adjudicado_negociado = Regex::new(
        r"Adjudicado e Homologado por CPF\s*(?P<cpf>[\d\.\-\*]+)\s*-\s*(?P<responsavel>[^,]+),?\s*para\s+(?P<fornecedor>[^,]+),\s*CNPJ\s*(?P<cnpj>[\d\.\-/]+),\s*melhor\s+lance:\s*R\$\s*(?P<melhor_lance>[\d,\.]+).*?valor\s+negociado:\s*R\$\s*(?P<valor_negociado>[\d,\.]+)"
    ).unwrap();

    let re_adjucado = Regex::new(
        r"Adjucado e Homologado por CPF\s*(?P<cpf>[\d\.\-\*]+)\s*-\s*(?P<responsavel>[^,]+),?\s*para\s+(?P<fornecedor>[^,]+),\s*CNPJ\s*(?P<cnpj>[\d\.\-/]+),\s*melhor\s+lance:\s*R\$\s*(?P<melhor_lance>[\d,\.]+)"
    ).unwrap();

    let re_adjudicado = Regex::new(
        r"Adjudicado e Homologado por CPF\s*(?P<cpf>[\d\.\-\*]+)\s*-\s*(?P<responsavel>[^,]+),?\s*para\s+(?P<fornecedor>[^,]+),\s*CNPJ\s*(?P<cnpj>[\d\.\-/]+),\s*melhor\s+lance:\s*R\$\s*(?P<melhor_lance>[\d,\.]+)"
    ).unwrap();

    let padroes_adjudicacao = vec![
        (&re_adjucado_negociado, true),
        (&re_adjudicado_negociado, true),
        (&re_adjucado, false),
        (&re_adjudicado, false),
    ];

    for (regex, tem_valor_negociado) in padroes_adjudicacao {
        for caps_adjudicado in regex.captures_iter(text) {
            let cnpj = caps_adjudicado.get(4).unwrap().as_str().trim();
            
            if cnpjs_processados.contains(cnpj) {
                continue;
            }
            cnpjs_processados.insert(cnpj.to_string());

            let melhor_lance = caps_adjudicado.get(5).unwrap().as_str().trim();
            let valor_adjudicado = if tem_valor_negociado {
                caps_adjudicado.get(6).unwrap().as_str().trim()
            } else {
                melhor_lance
            };

            let proposta = PropostaAdjudicada {
                item: extrair_item_do_contexto(text, cnpj),
                grupo: None,
                descricao: extrair_descricao_do_contexto(text, cnpj),
                quantidade: extrair_quantidade_do_contexto(text, cnpj),
                valor_estimado: extrair_valor_estimado_do_contexto(text, cnpj),
                valor_adjudicado: valor_adjudicado.to_string(),
                fornecedor: caps_adjudicado.get(3).unwrap().as_str().trim().to_string(),
                cnpj: cnpj.to_string(),
                melhor_lance: melhor_lance.to_string(),
                responsavel: caps_adjudicado.get(2).unwrap().as_str().trim().to_string(),
                cpf_responsavel: caps_adjudicado.get(1).unwrap().as_str().trim().to_string(),
                marca_fabricante: extrair_marca_fabricante_do_contexto(text, cnpj),
                modelo_versao: extrair_modelo_versao_do_contexto(text, cnpj),
                tipo_formato: "individual".to_string(),
            };

            if verbose {
                println!("✅ Proposta individual extraída - Item: {}, Fornecedor: {}, CNPJ: {}, Valor: R$ {}", 
                         proposta.item, proposta.fornecedor, proposta.cnpj, proposta.valor_adjudicado);
            }

            propostas.push(proposta);
        }
    }

    propostas
}

/// Extrai propostas no formato de grupo
fn extrair_propostas_grupo(text: &str, verbose: bool) -> Vec<PropostaAdjudicada> {
    let mut propostas = Vec::new();
    let mut cnpjs_processados = HashSet::new();

    // Padrão para formato de grupo
    let padrao_grupo = r"Item\s+(?P<item>\d+)\s+do\s+Grupo\s+G(?P<grupo>\d+)\s*-\s*(?P<descricao>[^\n]+)[\s\S]*?Quantidade:\s*(?P<quantidade>\d+)[\s\S]*?Valor\s+estimado:\s*R\$\s*(?P<valor>[\d,\.]+)[\s\S]*?Situação:\s*(?P<situacao>Adjudicado e Homologado)[\s\S]*?Adjudicado e Homologado por CPF[^-]+-\s*(?P<responsavel>[^,]+?)\s*para\s+(?P<fornecedor>[^,]+),\s*CNPJ\s*(?P<cnpj>[\d\.\-/]+),\s*melhor\s+lance:\s*R\$\s*(?P<melhor_lance>[\d,\.]+)";

    let re_grupo = Regex::new(padrao_grupo).unwrap();

    for caps in re_grupo.captures_iter(text) {
        let cnpj = caps.name("cnpj").unwrap().as_str().trim();
        let item = caps.name("item").unwrap().as_str().trim();
        let key = format!("{}-{}", item, cnpj);
        
        if cnpjs_processados.contains(&key) {
            continue;
        }
        cnpjs_processados.insert(key);

        let proposta = PropostaAdjudicada {
            item: item.to_string(),
            grupo: Some(format!("G{}", caps.name("grupo").unwrap().as_str())),
            descricao: caps.name("descricao").unwrap().as_str().trim().to_string(),
            quantidade: caps.name("quantidade").unwrap().as_str().trim().to_string(),
            valor_estimado: caps.name("valor").unwrap().as_str().trim().to_string(),
            valor_adjudicado: caps.name("melhor_lance").unwrap().as_str().trim().to_string(),
            fornecedor: caps.name("fornecedor").unwrap().as_str().trim().to_string(),
            cnpj: cnpj.to_string(),
            melhor_lance: caps.name("melhor_lance").unwrap().as_str().trim().to_string(),
            responsavel: caps.name("responsavel").unwrap().as_str().trim().to_string(),
            cpf_responsavel: extrair_cpf_do_responsavel(&caps.name("responsavel").unwrap().as_str()),
            marca_fabricante: "N/A".to_string(),
            modelo_versao: "N/A".to_string(),
            tipo_formato: "grupo".to_string(),
        };

        if verbose {
            println!("✅ Proposta de grupo extraída - Item: {}, Grupo: {}, Fornecedor: {}, CNPJ: {}, Valor: R$ {}", 
                     proposta.item, proposta.grupo.as_ref().unwrap(), proposta.fornecedor, proposta.cnpj, proposta.valor_adjudicado);
        }

        propostas.push(proposta);
    }

    propostas
}

/// Extrai CPF do responsável
fn extrair_cpf_do_responsavel(responsavel: &str) -> String {
    let re_cpf = Regex::new(r"(\*{3}\.\d{3}\.\*{3}-\*\d)").unwrap();
    if let Some(caps) = re_cpf.captures(responsavel) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Extrai item do contexto baseado no CNPJ
fn extrair_item_do_contexto(text: &str, cnpj: &str) -> String {
    let padrao = format!(r"Item\s+(\d+)[^#]*?{}", regex::escape(cnpj));
    let re = Regex::new(&padrao).unwrap();
    
    if let Some(caps) = re.captures(text) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Extrai descrição do contexto baseado no CNPJ
fn extrair_descricao_do_contexto(text: &str, cnpj: &str) -> String {
    let padrao = format!(r"Item\s+\d+[^#]*?([^#]*?){}", regex::escape(cnpj));
    let re = Regex::new(&padrao).unwrap();
    
    if let Some(caps) = re.captures(text) {
        let desc = caps.get(1).unwrap().as_str();
        desc.split('\n').next().unwrap_or("N/A").trim().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Extrai quantidade do contexto baseado no CNPJ
fn extrair_quantidade_do_contexto(text: &str, cnpj: &str) -> String {
    let padroes = vec![
        format!(r"Quantidade:\s*(\d+)[^#]*?{}", regex::escape(cnpj)),
        format!(r"Unidade\s+(\d+)[^#]*?{}", regex::escape(cnpj)),
    ];
    
    for padrao in padroes {
        let re = Regex::new(&padrao).unwrap();
        if let Some(caps) = re.captures(text) {
            return caps.get(1).unwrap().as_str().to_string();
        }
    }
    
    "N/A".to_string()
}

/// Extrai valor estimado do contexto baseado no CNPJ
fn extrair_valor_estimado_do_contexto(text: &str, cnpj: &str) -> String {
    let padroes = vec![
        format!(r"Valor\s+estimado:\s*R\$\s*([\d,\.]+)[^#]*?{}", regex::escape(cnpj)),
        format!(r"R\$\s*([\d,\.]+)Quantidade:[^#]*?{}", regex::escape(cnpj)),
    ];
    
    for padrao in padroes {
        let re = Regex::new(&padrao).unwrap();
        if let Some(caps) = re.captures(text) {
            return caps.get(1).unwrap().as_str().to_string();
        }
    }
    
    "N/A".to_string()
}

/// Extrai marca/fabricante do contexto baseado no CNPJ
fn extrair_marca_fabricante_do_contexto(text: &str, cnpj: &str) -> String {
    let padrao = format!(r"{}[\s\S]*?Proposta adjudicada[\s\S]*?Marca/Fabricante:\s*([^\n\r]+)", regex::escape(cnpj));
    let re = Regex::new(&padrao).unwrap();
    
    if let Some(caps) = re.captures(text) {
        return caps.get(1).unwrap().as_str().trim().to_string();
    }
    
    "N/A".to_string()
}

/// Extrai modelo/versão do contexto baseado no CNPJ
fn extrair_modelo_versao_do_contexto(text: &str, cnpj: &str) -> String {
    let padrao = format!(r"{}[\s\S]*?Proposta adjudicada[\s\S]*?Modelo/versão:\s*([^\n\r]+)", regex::escape(cnpj));
    let re = Regex::new(&padrao).unwrap();
    
    if let Some(caps) = re.captures(text) {
        return caps.get(1).unwrap().as_str().trim().to_string();
    }
    
    "N/A".to_string()
}

/// Converte string de valor para float
pub fn converter_valor_para_float(valor_str: &str) -> f64 {
    valor_str.replace(".", "")
        .replace(",", ".")
        .parse::<f64>()
        .unwrap_or(0.0)
}

/// Gera markdown a partir do relatório
fn gerar_markdown(relatorio: &RelatorioLicitacao) -> Result<String> {
    let mut markdown = String::new();
    
    // Cabeçalho
    markdown.push_str("---\n");
    markdown.push_str(&format!("gerado_em: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    markdown.push_str("ferramenta: PDF to Markdown Converter\n");
    markdown.push_str("---\n\n");
    
    // Título
    markdown.push_str("# RELATÓRIO DE LICITAÇÃO - PROPOSTAS ADJUDICADAS\n\n");
    
    // Informações gerais
    markdown.push_str("## Informações Gerais\n\n");
    markdown.push_str(&format!("- **UASG**: {}\n", relatorio.uasg));
    markdown.push_str(&format!("- **Pregão**: {}\n", relatorio.pregao));
    markdown.push_str(&format!("- **Processo**: {}\n", relatorio.processo));
    markdown.push_str(&format!("- **Data de Homologação**: {}\n", relatorio.data_homologacao));
    markdown.push_str(&format!("- **Responsável**: {}\n", relatorio.responsavel));
    markdown.push_str(&format!("- **Valor Total**: R$ {:.2}\n\n", relatorio.valor_total));
    
    // Tabela de propostas
    markdown.push_str("## Propostas Adjudicadas\n\n");
    
    // Verificar se há propostas por grupo
    let tem_grupos = relatorio.propostas.iter().any(|p| p.grupo.is_some());
    
    if tem_grupos {
        markdown.push_str("| Item | Grupo | Descrição | Quantidade | Valor Estimado | Valor Adjudicado | Fornecedor | CNPJ | Marca/Fabricante | Modelo/Versão |\n");
        markdown.push_str("|------|--------|-----------|------------|----------------|------------------|------------|------|------------------|---------------|\n");
    } else {
        markdown.push_str("| Item | Descrição | Quantidade | Valor Estimado | Valor Adjudicado | Fornecedor | CNPJ | Marca/Fabricante | Modelo/Versão |\n");
        markdown.push_str("|------|-----------|------------|----------------|------------------|------------|------|------------------|---------------|\n");
    }
    
    for proposta in &relatorio.propostas {
        if tem_grupos {
            markdown.push_str(&format!(
                "| {} | {} | {} | {} | R$ {} | R$ {} | {} | {} | {} | {} |\n",
                proposta.item,
                proposta.grupo.as_ref().unwrap_or(&"N/A".to_string()),
                proposta.descricao,
                proposta.quantidade,
                proposta.valor_estimado,
                proposta.valor_adjudicado,
                proposta.fornecedor,
                proposta.cnpj,
                proposta.marca_fabricante,
                proposta.modelo_versao
            ));
        } else {
            markdown.push_str(&format!(
                "| {} | {} | {} | R$ {} | R$ {} | {} | {} | {} | {} |\n",
                proposta.item,
                proposta.descricao,
                proposta.quantidade,
                proposta.valor_estimado,
                proposta.valor_adjudicado,
                proposta.fornecedor,
                proposta.cnpj,
                proposta.marca_fabricante,
                proposta.modelo_versao
            ));
        }
    }
    
    // Detalhes das propostas
    markdown.push_str("\n## Detalhes das Propostas\n\n");
    
    for proposta in &relatorio.propostas {
        let grupo_info = if let Some(grupo) = &proposta.grupo {
            format!(" ({}) ", grupo)
        } else {
            " ".to_string()
        };
        
        markdown.push_str(&format!("### Item {}{}- {}\n\n", proposta.item, grupo_info, proposta.descricao));
        markdown.push_str(&format!("- **Quantidade**: {}\n", proposta.quantidade));
        markdown.push_str(&format!("- **Valor Estimado**: R$ {}\n", proposta.valor_estimado));
        markdown.push_str(&format!("- **Valor Adjudicado**: R$ {}\n", proposta.valor_adjudicado));
        markdown.push_str(&format!("- **Fornecedor**: {}\n", proposta.fornecedor));
        markdown.push_str(&format!("- **CNPJ**: {}\n", proposta.cnpj));
        markdown.push_str(&format!("- **Melhor Lance**: R$ {}\n", proposta.melhor_lance));
        markdown.push_str(&format!("- **Responsável**: {}\n", proposta.responsavel));
        markdown.push_str(&format!("- **CPF Responsável**: {}\n", proposta.cpf_responsavel));
        markdown.push_str(&format!("- **Marca/Fabricante**: {}\n", proposta.marca_fabricante));
        markdown.push_str(&format!("- **Modelo/Versão**: {}\n\n", proposta.modelo_versao));
    }
    
    // Resumo estatístico
    markdown.push_str("## Resumo Estatístico\n\n");
    markdown.push_str(&format!("- **Total de Itens Adjudicados**: {}\n", relatorio.propostas.len()));
    markdown.push_str(&format!("- **Valor Total das Adjudicações**: R$ {:.2}\n", relatorio.valor_total));
    
    if !relatorio.propostas.is_empty() {
        let valor_medio = relatorio.valor_total / relatorio.propostas.len() as f64;
        markdown.push_str(&format!("- **Valor Médio por Item**: R$ {:.2}\n", valor_medio));
    }
    
    Ok(markdown)
}

/// Extrai UASG do texto
fn extrair_uasg(text: &str) -> String {
    let re = Regex::new(r"UASG\s*(\d+)").unwrap();
    if let Some(caps) = re.captures(text) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Extrai pregão do texto
fn extrair_pregao(text: &str) -> String {
    let re = Regex::new(r"PREGÃO\s*(\d+/\d+)").unwrap();
    if let Some(caps) = re.captures(text) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Extrai processo do texto
fn extrair_processo(text: &str) -> String {
    let re = Regex::new(r"Processo\s*n[ºo°]?\s*(\d+)").unwrap();
    if let Some(caps) = re.captures(text) {
        caps.get(1).unwrap().as_str().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Extrai data de homologação do texto
fn extrair_data_homologacao(text: &str) -> String {
    let re = Regex::new(r"Às\s*([\d:]+)\s*horas\s*do\s*dia\s*([\d]+)\s*de\s*(\w+)\s*do\s*ano\s*de\s*([\d]+)").unwrap();
    if let Some(caps) = re.captures(text) {
        format!("Às {} horas do dia {} de {} do ano de {}", 
                caps.get(1).unwrap().as_str(),
                caps.get(2).unwrap().as_str(),
                caps.get(3).unwrap().as_str(),
                caps.get(4).unwrap().as_str())
    } else {
        "N/A".to_string()
    }
}

/// Extrai responsável do texto
fn extrair_responsavel(text: &str) -> String {
    let re = Regex::new(r"HOMOLOGA\s*a\s*adjudicação.*?([A-Z][A-Z\s]+),").unwrap();
    if let Some(caps) = re.captures(text) {
        caps.get(1).unwrap().as_str().trim().to_string()
    } else {
        "N/A".to_string()
    }
}

/// Salva JSON consolidado
pub fn salvar_json_consolidado(
    propostas: &[PropostaConsolidada], 
    output_dir: &Path, 
    _nome_arquivo: &str, 
    verbose: bool
) -> Result<()> {
    let valor_total_geral: f64 = propostas.iter()
        .map(|p| converter_valor_para_float(&p.valor_adjudicado))
        .sum();
    
    // Agrupar propostas por UASG + Pregão + Processo
    let mut licitacoes: HashMap<String, LicitacaoConsolidada> = HashMap::new();
    
    for proposta in propostas {
        let chave = format!("{}-{}-{}", proposta.uasg, proposta.pregao, proposta.processo);
        
        let licitacao = licitacoes.entry(chave).or_insert_with(|| LicitacaoConsolidada {
            uasg: proposta.uasg.clone(),
            pregao: proposta.pregao.clone(),
            processo: proposta.processo.clone(),
            total_propostas: 0,
            valor_total: 0.0,
            propostas: Vec::new(),
        });
        
        licitacao.propostas.push(proposta.clone());
        licitacao.total_propostas += 1;
        licitacao.valor_total += converter_valor_para_float(&proposta.valor_adjudicado);
    }
    
    let data_geracao = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let mut arquivos_salvos = 0;
    
    // Salvar um arquivo JSON para cada licitação
    for (chave, licitacao) in &licitacoes {
        let nome_arquivo_licitacao = format!("licitacao_{}.json", 
            chave.replace("/", "_").replace(" ", "_"));
        
        let json_licitacao = serde_json::json!({
            "data_geracao": data_geracao,
            "uasg": licitacao.uasg,
            "pregao": licitacao.pregao,
            "processo": licitacao.processo,
            "total_propostas": licitacao.total_propostas,
            "valor_total": licitacao.valor_total,
            "propostas": licitacao.propostas
        });
        
        let json_path = output_dir.join(&nome_arquivo_licitacao);
        let json_content = serde_json::to_string_pretty(&json_licitacao)
            .context("Erro ao serializar JSON da licitação")?;
        
        fs::write(&json_path, json_content)
            .context(format!("Erro ao salvar arquivo JSON: {}", nome_arquivo_licitacao))?;
        
        arquivos_salvos += 1;
        
        if verbose {
            println!("📄 JSON licitação salvo: {:?} ({} propostas, R$ {:.2})", 
                     json_path, licitacao.total_propostas, licitacao.valor_total);
        }
    }
    
    // Salvar também um arquivo resumo geral
    let resumo_geral = serde_json::json!({
        "data_geracao": data_geracao,
        "total_licitacoes": licitacoes.len(),
        "total_propostas": propostas.len(),
        "valor_total_geral": valor_total_geral,
        "arquivos_gerados": licitacoes.keys().map(|k| format!("licitacao_{}.json", 
            k.replace("/", "_").replace(" ", "_"))).collect::<Vec<_>>()
    });
    
    let resumo_path = output_dir.join("resumo_geral.json");
    let resumo_content = serde_json::to_string_pretty(&resumo_geral)
        .context("Erro ao serializar resumo geral")?;
    
    fs::write(&resumo_path, resumo_content)
        .context("Erro ao salvar arquivo de resumo geral")?;
    
    if verbose {
        println!("📊 Resumo geral:");
        println!("   - {} arquivos JSON de licitações salvos", arquivos_salvos);
        println!("   - {} propostas totais processadas", propostas.len());
        println!("   - Valor total geral: R$ {:.2}", valor_total_geral);
        println!("📄 Resumo geral salvo em: {:?}", resumo_path);
    }
    
    Ok(())
} 