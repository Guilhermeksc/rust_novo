import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { invoke } from '@tauri-apps/api/core';

interface JsonFileInfo {
  file_name: string;
  file_path: string;
  file_size: number;
  modified_timestamp: number;
  propostas_count?: number;
  error?: string;
}

interface SicafData {
  cnpj: string;
  empresa: string;
  nome_fantasia?: string;
  situacao_cadastro?: string;
  data_vencimento?: string;
  cep?: string;
  endereco?: string;
  municipio?: string;
  uf?: string;
  telefone?: string;
  email?: string;
  cpf_responsavel?: string;
  nome_responsavel?: string;
}

interface ProcessingSicafResult {
  success: boolean;
  message: string;
  processed_count: number;
  sicaf_data: SicafData[];
  session_id?: string;
}

@Component({
    selector: 'app-dados-empresas',
    standalone: true,
    imports: [CommonModule, FormsModule],
    templateUrl: './dados-empresas.component.html',
    styleUrls: ['./dados-empresas.component.css']
})
export class DadosEmpresasComponent implements OnInit {
  jsonFiles: JsonFileInfo[] = [];
  selectedJsonFile: string = '';
  sicafData: SicafData[] = [];
  cnpjRegistrados: Set<string> = new Set();
  
  // Estados de processamento
  isProcessing: boolean = false;
  processedCount: number = 0;
  pdfCount: number = 0;
  
  // Logs
  logs: string[] = [];
  
  constructor() {}

  ngOnInit() {
    this.initializeSystem();
  }

  async initializeSystem() {
    try {
      // Inicializar estrutura Database
      await invoke<string>('initialize_database_structure');
      this.addLog('Estrutura Database inicializada');
      
      // Carregar dados
      await this.loadJsonFiles();
      await this.loadPdfCount();
    } catch (error) {
      this.addLog('Erro ao inicializar sistema: ' + error);
    }
  }

  async loadJsonFiles() {
    try {
      // Carregar arquivos JSON do diretório de output
      const outputDir = await invoke<string>('get_output_directory');
      const jsonFiles = await invoke<JsonFileInfo[]>('list_json_files', { directory: outputDir });
      
      // Filtrar apenas arquivos que começam com 'licitacao_'
      this.jsonFiles = jsonFiles.filter(file => 
        file.file_name.startsWith('licitacao_') && !file.error
      );
      
      this.addLog('Arquivos JSON carregados: ' + this.jsonFiles.length);
    } catch (error) {
      this.addLog('Erro ao carregar arquivos JSON: ' + error);
    }
  }

  async loadPdfCount() {
    try {
      const sicafDir = await invoke<string>('get_sicaf_directory');
      const pdfFiles = await invoke<string[]>('list_pdf_files', { directory: sicafDir });
      this.pdfCount = pdfFiles.length;
      this.addLog(`Arquivos PDF encontrados: ${this.pdfCount}`);
    } catch (error) {
      this.addLog('Erro ao contar PDFs: ' + error);
    }
  }

  async onJsonFileSelected() {
    if (!this.selectedJsonFile) {
      this.sicafData = [];
      this.cnpjRegistrados.clear();
      return;
    }

    try {
      // Carregar dados do JSON selecionado
      const jsonData = await invoke<any>('read_json_file', { filePath: this.selectedJsonFile });
      
      if (jsonData.propostas && Array.isArray(jsonData.propostas)) {
        // Extrair CNPJs únicos das propostas
        const cnpjSet = new Set<string>();
        const sicafDataTemp: SicafData[] = [];
        
        jsonData.propostas.forEach((proposta: any) => {
          if (proposta.cnpj && !cnpjSet.has(proposta.cnpj)) {
            cnpjSet.add(proposta.cnpj);
            sicafDataTemp.push({
              cnpj: proposta.cnpj,
              empresa: proposta.fornecedor || 'N/A',
              nome_fantasia: proposta.nome_fantasia,
              situacao_cadastro: 'Não verificado',
              endereco: proposta.endereco
            });
          }
        });
        
        this.sicafData = sicafDataTemp;
        this.cnpjRegistrados = cnpjSet;
        this.addLog(`Arquivo JSON carregado: ${jsonData.propostas.length} propostas, ${cnpjSet.size} CNPJs únicos`);
      }
    } catch (error) {
      this.addLog('Erro ao carregar arquivo JSON: ' + error);
    }
  }

  async openSicafFolder() {
    try {
      const sicafDir = await invoke<string>('get_sicaf_directory');
      await invoke('open_folder', { path: sicafDir });
    } catch (error) {
      this.addLog('Erro ao abrir pasta SICAF: ' + error);
    }
  }

  async openResultsFolder() {
    try {
      const outputDir = await invoke<string>('get_output_directory');
      await invoke('open_folder', { path: outputDir });
      this.addLog('Pasta de Resultados aberta para adicionar arquivos JSON');
    } catch (error) {
      this.addLog('Erro ao abrir pasta de Resultados: ' + error);
    }
  }

  async processSicafPdfs() {
    if (this.isProcessing) return;
    
    this.isProcessing = true;
    this.processedCount = 0;
    this.addLog('Iniciando processamento de PDFs SICAF...');
    
    try {
      const result = await invoke<ProcessingSicafResult>('process_sicaf_pdfs', {
        verbose: true
      });
      
      if (result.success) {
        this.processedCount = result.processed_count;
        this.addLog(`Processamento concluído: ${result.processed_count} PDFs processados`);
        
        // Atualizar dados SICAF com os dados processados
        this.updateSicafDataWithProcessedData(result.sicaf_data);
        
        // Recarregar contagem de PDFs
        this.loadPdfCount();
      } else {
        this.addLog('Erro no processamento: ' + result.message);
      }
    } catch (error) {
      this.addLog('Erro ao processar PDFs: ' + error);
    } finally {
      this.isProcessing = false;
    }
  }

  private updateSicafDataWithProcessedData(processedData: SicafData[]) {
    // Criar um mapa dos dados processados por CNPJ
    const processedMap = new Map<string, SicafData>();
    processedData.forEach(data => {
      processedMap.set(data.cnpj, data);
    });
    
    // Atualizar dados existentes
    this.sicafData = this.sicafData.map(item => {
      const processed = processedMap.get(item.cnpj);
      if (processed) {
        return {
          ...item,
          ...processed,
          situacao_cadastro: 'Verificado'
        };
      }
      return item;
    });
    
    this.addLog(`Dados SICAF atualizados para ${processedData.length} empresas`);
  }

  async refreshAll() {
    this.addLog('Atualizando dados...');
    await this.loadJsonFiles();
    await this.loadPdfCount();
    
    if (this.selectedJsonFile) {
      await this.onJsonFileSelected();
    }
    
    this.addLog('Atualização concluída');
  }

  isCnpjRegistered(cnpj: string): boolean {
    return this.cnpjRegistrados.has(cnpj);
  }

  getSicafStatus(cnpj: string): string {
    const sicafItem = this.sicafData.find(item => item.cnpj === cnpj);
    return sicafItem?.situacao_cadastro || 'Não verificado';
  }

  copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      this.addLog(`CNPJ ${text} copiado para área de transferência`);
    }).catch(err => {
      this.addLog('Erro ao copiar: ' + err);
    });
  }

  private addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    this.logs.unshift(`[${timestamp}] ${message}`);
    
    // Manter apenas os últimos 50 logs
    if (this.logs.length > 50) {
      this.logs = this.logs.slice(0, 50);
    }
  }

  clearLogs() {
    this.logs = [];
  }

  get verifiedCount(): number {
    return this.sicafData.filter(item => item.situacao_cadastro === 'Verificado').length;
  }

  get notVerifiedCount(): number {
    return this.sicafData.filter(item => item.situacao_cadastro === 'Não verificado').length;
  }
}
