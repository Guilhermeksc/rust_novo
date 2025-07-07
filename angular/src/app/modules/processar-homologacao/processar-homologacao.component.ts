import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { SelecionarPastaComponent } from './selecionar-pasta/selecionar-pasta.component';
import { RelacaoPdfComponent } from './relacao-pdf/relacao-pdf.component';


interface ProcessingResult {
  success: boolean;
  message: string;
  file_path?: string;
  extracted_data?: any;
  propostas: any[];
  total_processed: number;
  json_file_path?: string;
  session_id?: string;
}

interface ProcessingStatus {
  is_processing: boolean;
  current_file?: string;
  processed_files: number;
  total_files: number;
  errors: string[];
  progress_percentage: number;
}

interface JsonFileInfo {
  file_name: string;
  file_path: string;
  file_size: number;
  modified_timestamp: number;
  data_geracao?: string;
  pregao?: string;
  processo?: string;
  uasg?: string;
  total_propostas?: number;
  valor_total?: number;
  propostas_count?: number;
  error?: string;
}

interface PropostaData {
  cnpj: string;
  descricao: string;
  fornecedor: string;
  grupo?: string;
  item: string;
  marca_fabricante: string;
  melhor_lance: string;
  modelo_versao: string;
  pregao: string;
  processo: string;
  quantidade: string;
  responsavel: string;
  tipo_formato: string;
  uasg: string;
  valor_adjudicado: string;
  valor_estimado: string;
}

interface JsonData {
  data_geracao: string;
  pregao: string;
  processo: string;
  propostas: PropostaData[];
  total_propostas: number;
  uasg: string;
  valor_total: number;
}

interface ProcessingLog {
  timestamp: string;
  message: string;
  log_type: string;
  session_id?: string;
}

interface AppConfig {
  last_input_directory?: string;
  last_output_directory?: string;
  verbose: boolean;
  processing_logs: ProcessingLog[];
  max_logs: number;
  created_at: string;
  updated_at: string;
}

interface ConfigResult {
  success: boolean;
  message: string;
  config?: AppConfig;
}

interface PdfFileInfo {
  file_name: string;
  file_path: string;
  file_size: number;
  modified_timestamp: number;
}

@Component({
    selector: 'app-processar-homologacao',
    standalone: true,
    imports: [
        CommonModule, 
        FormsModule,
        SelecionarPastaComponent,
        RelacaoPdfComponent
    ],
    templateUrl: './processar-homologacao.component.html',
    styleUrls: ['./processar-homologacao.component.css']
})
export class ProcessarHomologacaoComponent implements OnInit, OnDestroy {
  // PDF Processing variables
  selectedFile: string = "";
  pdfDirectory: string = "";
  outputDirectory: string = "";
  verbose: boolean = false;
  processing: boolean = false;
  processingResult: ProcessingResult | null = null;
  processingStatus: ProcessingStatus | null = null;
  currentSessionId: string = "";
  
  // Progress monitoring
  progressInterval: any;
  progressLogs: {message: string, timestamp: Date, type: 'info' | 'success' | 'error' | 'progress'}[] = [];
  totalProposals: number = 0;
  
  // Results system
  availableJsonFiles: JsonFileInfo[] = [];
  selectedJsonFile: JsonFileInfo | null = null;
  jsonData: JsonData | null = null;
  loadingJsonFiles: boolean = false;
  loadingJsonData: boolean = false;
  
  // UI state
  activeTab: 'processing' | 'logs' | 'results' = 'processing';
  
  // Configuration management
  appConfig: AppConfig | null = null;
  configLoaded: boolean = false;
  persistentLogs: ProcessingLog[] = [];
  showPersistentLogs: boolean = false;
  isRepairingConfig: boolean = false; // NEW: Flag to prevent repair loops

  // PDF Navigator - NEW
  availablePdfFiles: PdfFileInfo[] = [];
  selectedPdfFile: PdfFileInfo | null = null;
  loadingPdfFiles: boolean = false;
  showPdfViewer: boolean = false;
  pdfViewerUrl: string = '';

  // Directory Management - NEW
  isChangingDirectories: boolean = false;

  ngOnInit(): void {
    this.initializeConfiguration();
  }

  async initializeConfiguration(): Promise<void> {
    try {
      this.addLog('🔧 Inicializando aplicação...', 'info');
      
      // First, try to initialize the application
      let initializationSuccess = false;
      try {
        const initResult = await invoke<ConfigResult>('initialize_application');
        if (initResult.success) {
          this.addLog('✅ Aplicação inicializada com sucesso', 'success');
          if (initResult.config) {
            this.appConfig = initResult.config;
            this.configLoaded = true;
            initializationSuccess = true;
          }
        }
      } catch (error: any) {
        console.error('Error initializing application:', error);
        
        // Check if it's a config corruption issue and auto-repair
        const isConfigError = error?.details?.includes('licitacao360_config.json') || 
                             error?.message?.includes('UTF-8') || 
                             error?.message?.includes('deserializar') ||
                             error?.message?.includes('trailing characters');
        
        if (isConfigError) {
          this.addLog('⚠️ Configuração corrompida detectada, executando reparo automático...', 'error');
          try {
            await this.performAutomaticRepair();
            // Try initialization again after repair
            const retryResult = await invoke<ConfigResult>('initialize_application');
            if (retryResult.success) {
              this.addLog('✅ Aplicação inicializada com sucesso após reparo', 'success');
              if (retryResult.config) {
                this.appConfig = retryResult.config;
                this.configLoaded = true;
                initializationSuccess = true;
              }
            }
          } catch (repairError) {
            console.error('Error during automatic repair:', repairError);
            this.addLog('❌ Falha no reparo automático', 'error');
          }
        } else {
          this.addLog('⚠️ Erro na inicialização, tentando carregar configuração existente...', 'error');
        }
      }
      
      // Load configuration if not already loaded
      if (!this.configLoaded) {
        await this.loadConfiguration();
      }
      
      // Get directory information (only if initialization was successful)
      if (initializationSuccess) {
        try {
          const dirInfo = await invoke<any>('get_app_directories_info');
          this.addLog('📂 Informações dos diretórios:', 'info');
          this.addLog(`  • Home: ${dirInfo.home_directory}`, 'info');
          this.addLog(`  • Config: ${dirInfo.config_directory}`, 'info');
          this.addLog(`  • PDF padrão: ${dirInfo.default_pdf_directory}`, 'info');
          this.addLog(`  • Saída padrão: ${dirInfo.default_output_directory}`, 'info');
          this.addLog(`  • Arquivo config existe: ${dirInfo.config_file_exists}`, 'info');
        } catch (error) {
          console.error('Error getting directory info:', error);
        }
      }
      
      // Set directories from config or defaults
      if (this.appConfig?.last_input_directory) {
        this.pdfDirectory = this.appConfig.last_input_directory;
        this.addLog(`📁 Usando pasta PDF salva: ${this.pdfDirectory}`, 'success');
      } else {
        try {
          this.pdfDirectory = await invoke<string>('get_default_pdf_directory');
          this.addLog(`📁 Usando pasta PDF padrão: ${this.pdfDirectory}`, 'info');
        } catch (error) {
          this.addLog('❌ Erro ao obter pasta PDF padrão', 'error');
        }
      }
      
      if (this.appConfig?.last_output_directory) {
        this.outputDirectory = this.appConfig.last_output_directory;
        this.addLog(`📤 Usando pasta resultados salva: ${this.outputDirectory}`, 'success');
      } else {
        try {
          this.outputDirectory = await invoke<string>('get_default_output_directory');
          this.addLog(`📤 Usando pasta resultados padrão: ${this.outputDirectory}`, 'info');
        } catch (error) {
          this.addLog('❌ Erro ao obter pasta de resultados padrão', 'error');
        }
      }
      
      // Load verbose setting
      if (this.appConfig?.verbose !== undefined) {
        this.verbose = this.appConfig.verbose;
        this.addLog(`🔍 Modo verbose: ${this.verbose ? 'ativado' : 'desativado'}`, 'info');
      }
      
      // Load persistent logs
      if (this.appConfig?.processing_logs) {
        this.persistentLogs = this.appConfig.processing_logs;
        this.addLog(`📝 ${this.persistentLogs.length} logs históricos carregados`, 'info');
      }
      
      // Load PDF files list
      await this.loadPdfFilesList();
      
      this.addLog('✅ Configuração completa:', 'success');
      this.addLog(`  • PDF: ${this.pdfDirectory}`, 'info');
      this.addLog(`  • Resultados: ${this.outputDirectory}`, 'info');
    } catch (error) {
      console.error('Error initializing configuration:', error);
      this.addLog('❌ Erro ao configurar aplicação', 'error');
    }
  }

  async performAutomaticRepair(): Promise<void> {
    if (this.isRepairingConfig) {
      console.warn('Repair already in progress, skipping...');
      return;
    }
    
    this.isRepairingConfig = true;
    
    try {
      this.addLog('🔧 Executando reparo automático da configuração...', 'info');
      
      const result = await invoke<ConfigResult>('debug_and_repair_config');
      
      if (result.success) {
        this.addLog('✅ Reparo automático executado com sucesso', 'success');
        
        // Log repair details without overwhelming the UI
        const lines = result.message.split('\n').slice(0, 10); // Only first 10 lines
        for (const line of lines) {
          if (line.trim()) {
            this.addLog(`  ${line.trim()}`, 'info');
          }
        }
        
        if (result.message.split('\n').length > 10) {
          this.addLog('  ... (detalhes completos disponíveis no console)', 'info');
          console.log('Full repair details:', result.message);
        }
      } else {
        this.addLog('❌ Falha no reparo automático', 'error');
        console.error('Repair failed:', result.message);
      }
    } catch (error) {
      console.error('Error during automatic repair:', error);
      this.addLog('❌ Erro crítico no reparo automático', 'error');
    } finally {
      this.isRepairingConfig = false;
    }
  }

  async repairConfiguration(): Promise<void> {
    try {
      this.addLog('🔧 Detectado problema na configuração, tentando reparar...', 'info');
      
      const result = await invoke<ConfigResult>('debug_and_repair_config');
      
      if (result.success) {
        this.addLog('✅ Configuração reparada com sucesso', 'success');
        
        // Log the repair details
        const lines = result.message.split('\n');
        for (const line of lines) {
          if (line.trim()) {
            this.addLog(`  ${line.trim()}`, 'info');
          }
        }
        
        // Reload configuration after repair
        await this.loadConfiguration();
      } else {
        this.addLog('❌ Falha ao reparar configuração', 'error');
      }
    } catch (error) {
      console.error('Error repairing configuration:', error);
      this.addLog('❌ Erro crítico ao reparar configuração', 'error');
    }
  }

  async loadConfiguration(): Promise<void> {
    try {
      this.appConfig = await invoke<AppConfig>('load_app_config');
      this.configLoaded = true;
    } catch (error: any) {
      console.error('Error loading configuration:', error);
      this.configLoaded = false;
      
      // Check if it's a UTF-8 error or deserialization error (corrupted config)
      const isUtf8Error = error?.message?.includes('UTF-8') || error?.message?.includes('stream did not contain valid UTF-8');
      const isDeserializationError = error?.details?.includes('licitacao360_config.json') && 
          (error?.message?.includes('deserializar') || error?.message?.includes('trailing characters'));
      
      if (isUtf8Error || isDeserializationError) {
        if (isUtf8Error) {
          this.addLog('⚠️ Arquivo de configuração contém dados UTF-8 inválidos', 'error');
        } else {
          this.addLog('⚠️ Arquivo de configuração corrompido detectado', 'error');
        }
        await this.repairConfiguration();
      } else {
        this.addLog('❌ Erro ao carregar configuração: ' + (error?.message || 'Erro desconhecido'), 'error');
      }
    }
  }

  async saveConfiguration(): Promise<void> {
    if (!this.appConfig) return;
    
    try {
      await invoke<ConfigResult>('save_app_config', { config: this.appConfig });
    } catch (error) {
      console.error('Error saving configuration:', error);
    }
  }

  async updateDirectoriesConfig(): Promise<void> {
    try {
      this.addLog(`🔧 Salvando configuração de pastas...`, 'info');
      this.addLog(`  • PDF: ${this.pdfDirectory}`, 'info');
      this.addLog(`  • Resultados: ${this.outputDirectory}`, 'info');
      
      const result = await invoke<ConfigResult>('update_config_directories', {
        input_dir: this.pdfDirectory || null,
        output_dir: this.outputDirectory || null
      });
      
      if (result.success) {
        this.addLog(`✅ Configuração salva com sucesso`, 'success');
        // Reload configuration to verify it was saved
        await this.loadConfiguration();
        
        // Verify the paths were saved correctly
        if (this.appConfig?.last_input_directory === this.pdfDirectory && 
            this.appConfig?.last_output_directory === this.outputDirectory) {
          this.addLog(`✅ Paths verificados e persistidos corretamente`, 'success');
        } else {
          this.addLog(`⚠️ Possível problema na persistência dos paths`, 'error');
          this.addLog(`  • Salvo PDF: ${this.appConfig?.last_input_directory}`, 'info');
          this.addLog(`  • Atual PDF: ${this.pdfDirectory}`, 'info');
          this.addLog(`  • Salvo Resultados: ${this.appConfig?.last_output_directory}`, 'info');
          this.addLog(`  • Atual Resultados: ${this.outputDirectory}`, 'info');
        }
      } else {
        this.addLog(`❌ Erro ao salvar configuração: ${result.message}`, 'error');
      }
    } catch (error) {
      console.error('Error updating directories configuration:', error);
      this.addLog(`❌ Erro crítico ao atualizar configuração: ${error}`, 'error');
    }
  }

  async updateVerboseConfig(): Promise<void> {
    try {
      await invoke<ConfigResult>('update_config_verbose', {
        verbose: this.verbose
      });
      
      // Reload configuration
      await this.loadConfiguration();
    } catch (error) {
      console.error('Error updating verbose configuration:', error);
    }
  }

  async addPersistentLog(message: string, type: 'info' | 'success' | 'error' | 'progress'): Promise<void> {
    // Skip persistent logging during config repair to avoid infinite loops
    if (this.isRepairingConfig) {
      console.log('Skipping persistent log during repair:', message);
      return;
    }
    
    try {
      await invoke<ConfigResult>('add_config_log', {
        message,
        logType: type,
        sessionId: this.currentSessionId || null
      });
      
      // Reload configuration to get updated logs
      await this.loadConfiguration();
      if (this.appConfig?.processing_logs) {
        this.persistentLogs = this.appConfig.processing_logs;
      }
    } catch (error: any) {
      console.error('Error adding persistent log:', error);
      
      // Don't try to add another persistent log to avoid infinite loops
      // Just log to console and add to local logs only
      console.warn('Persistent logging disabled due to configuration error');
      
      // Check if it's a UTF-8 error or config corruption issue
      const isUtf8Error = error?.message?.includes('UTF-8') || error?.message?.includes('stream did not contain valid UTF-8');
      const isDeserializationError = error?.details?.includes('licitacao360_config.json') && 
          (error?.message?.includes('deserializar') || error?.message?.includes('trailing characters'));
      
      if (isUtf8Error || isDeserializationError) {
        // Don't add log here to avoid recursion, but trigger repair if not already repairing
        console.warn('Configuration file appears corrupted (UTF-8 or JSON issue)');
        
        // Only trigger automatic repair if we're not already repairing
        if (!this.isRepairingConfig) {
          console.log('Triggering automatic config repair...');
          setTimeout(() => this.performAutomaticRepair(), 100); // Delay to avoid stack overflow
        }
      }
    }
  }

  async clearPersistentLogs(): Promise<void> {
    try {
      await invoke<ConfigResult>('clear_config_logs');
      this.persistentLogs = [];
      this.addLog('🗑️ Histórico de logs limpo', 'success');
    } catch (error) {
      console.error('Error clearing persistent logs:', error);
      this.addLog('❌ Erro ao limpar histórico de logs', 'error');
    }
  }

  togglePersistentLogs(): void {
    this.showPersistentLogs = !this.showPersistentLogs;
  }

  formatPersistentLogTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleString('pt-BR');
  }

  getPersistentLogClass(type: string): string {
    switch (type) {
      case 'success': return 'log-success';
      case 'error': return 'log-error';
      case 'progress': return 'log-progress';
      default: return 'log-info';
    }
  }

  async selectFile(): Promise<void> {
    try {
      const selected = await open({
        multiple: false,
        defaultPath: this.pdfDirectory,
        filters: [
          {
            name: 'PDF Files',
            extensions: ['pdf']
          }
        ]
      });
      
      if (selected && typeof selected === 'string') {
        this.selectedFile = selected;
        this.addLog(`📄 Arquivo selecionado: ${selected}`, 'info');
      }
    } catch (error) {
      console.error('Error selecting file:', error);
      this.addLog('❌ Erro ao selecionar arquivo', 'error');
    }
  }

  async selectDirectory(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        defaultPath: this.pdfDirectory
      });
      
      if (selected && typeof selected === 'string') {
        this.pdfDirectory = selected;
        this.addLog(`📁 Pasta selecionada: ${selected}`, 'info');
        
        // Save directory configuration
        await this.updateDirectoriesConfig();
      }
    } catch (error) {
      console.error('Error selecting directory:', error);
      this.addLog('❌ Erro ao selecionar pasta', 'error');
    }
  }

  async selectOutputDirectory(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        defaultPath: this.outputDirectory
      });
      
      if (selected && typeof selected === 'string') {
        this.outputDirectory = selected;
        this.addLog(`📤 Pasta de saída selecionada: ${selected}`, 'info');
        
        // Save directory configuration
        await this.updateDirectoriesConfig();
      }
    } catch (error) {
      console.error('Error selecting output directory:', error);
      this.addLog('❌ Erro ao selecionar pasta de saída', 'error');
    }
  }

  async processFile(): Promise<void> {
    if (!this.selectedFile || !this.outputDirectory) {
      this.addLog('❌ Arquivo ou pasta de saída não selecionados', 'error');
      return;
    }

    this.processing = true;
    this.processingResult = null;
    this.progressLogs = [];
    this.totalProposals = 0;
    this.currentSessionId = Date.now().toString();

    this.addLog('🔄 Iniciando processamento do arquivo...', 'info');

    try {
      const result = await invoke<ProcessingResult>('process_pdf_file', {
        file_path: this.selectedFile,
        output_dir: this.outputDirectory,
        verbose: this.verbose
      });

      if (result.session_id) {
        this.currentSessionId = result.session_id;
        this.startProgressMonitoring(result.session_id);
      }

      this.processingResult = result;
      this.totalProposals = result.propostas.length;
      
      if (result.success) {
        this.addLog(`✅ Arquivo processado com sucesso! ${result.propostas.length} propostas encontradas`, 'success');
        if (result.json_file_path) {
          this.addLog(`📄 Arquivo JSON salvo em: ${result.json_file_path}`, 'success');
        }
      } else {
        this.addLog(`❌ Erro no processamento: ${result.message}`, 'error');
      }
    } catch (error) {
      console.error('Error processing file:', error);
      this.addLog('❌ Erro durante o processamento', 'error');
      this.processingResult = {
        success: false,
        message: `Erro durante o processamento: ${error}`,
        propostas: [],
        total_processed: 0
      };
    } finally {
      this.processing = false;
      this.stopProgressMonitoring();
    }
  }

  async processDirectory(): Promise<void> {
    if (!this.pdfDirectory || !this.outputDirectory) {
      this.addLog('❌ Pasta ou pasta de saída não selecionadas', 'error');
      return;
    }

    this.processing = true;
    this.processingResult = null;
    this.progressLogs = [];
    this.totalProposals = 0;
    this.currentSessionId = Date.now().toString();

    this.addLog('🔄 Iniciando processamento da pasta...', 'info');

    try {
      const result = await invoke<ProcessingResult>('process_pdf_directory', {
        input_dir: this.pdfDirectory,
        output_dir: this.outputDirectory,
        verbose: this.verbose,
        session_id: this.currentSessionId
      });

      if (result.session_id) {
        this.currentSessionId = result.session_id;
        this.startProgressMonitoring(result.session_id);
      }

      this.processingResult = result;
      this.totalProposals = result.propostas.length;
      
      if (result.success) {
        this.addLog(`✅ Pasta processada com sucesso! ${result.total_processed} arquivos processados`, 'success');
        this.addLog(`📊 Total de propostas: ${result.propostas.length}`, 'success');
        if (result.json_file_path) {
          this.addLog(`📄 Arquivo JSON salvo em: ${result.json_file_path}`, 'success');
        }
      } else {
        this.addLog(`❌ Erro no processamento: ${result.message}`, 'error');
      }
    } catch (error) {
      console.error('Error processing directory:', error);
      this.addLog('❌ Erro durante o processamento', 'error');
      this.processingResult = {
        success: false,
        message: `Erro durante o processamento: ${error}`,
        propostas: [],
        total_processed: 0
      };
    } finally {
      this.processing = false;
      this.stopProgressMonitoring();
    }
  }

  async startProgressMonitoring(sessionId: string): Promise<void> {
    this.addLog('📊 Monitoramento de progresso iniciado', 'info');
    
    this.progressInterval = setInterval(async () => {
      try {
        const status = await invoke<ProcessingStatus>('get_processing_status', {
          session_id: sessionId
        });
        
        this.processingStatus = status;
        
        if (status.current_file) {
          this.addLog(`📄 Processando: ${status.current_file} (${status.processed_files}/${status.total_files})`, 'progress');
        }
        
        if (status.errors && status.errors.length > 0) {
          for (const error of status.errors) {
            this.addLog(`❌ ${error}`, 'error');
          }
        }
        
        // Se o processamento terminou
        if (!status.is_processing) {
          this.stopProgressMonitoring();
          this.addLog('✅ Processamento concluído', 'success');
          
          // Limpar estado de processamento
          try {
            await invoke('clear_processing_state', { session_id: sessionId });
          } catch (error) {
            console.warn('Erro ao limpar estado de processamento:', error);
          }
        }
      } catch (error) {
        console.error('Error getting processing status:', error);
        this.stopProgressMonitoring();
      }
    }, 500);
  }

  stopProgressMonitoring(): void {
    if (this.progressInterval) {
      clearInterval(this.progressInterval);
      this.progressInterval = null;
    }
  }

  ngOnDestroy(): void {
    this.stopProgressMonitoring();
  }

  addLog(message: string, type: 'info' | 'success' | 'error' | 'progress' = 'info'): void {
    const logEntry = {
      message,
      timestamp: new Date(),
      type
    };
    
    // Avoid duplicate progress logs
    if (type === 'progress') {
      const lastLog = this.progressLogs[this.progressLogs.length - 1];
      if (lastLog && lastLog.type === 'progress' && lastLog.message === message) {
        return;
      }
    }
    
    this.progressLogs.push(logEntry);
    
    // Keep only last 100 logs
    if (this.progressLogs.length > 100) {
      this.progressLogs = this.progressLogs.slice(-100);
    }
    
    // Save important logs persistently (not progress logs to avoid spam)
    // Skip persistent logging during config repair to avoid infinite loops
    if (type !== 'progress' && !this.isRepairingConfig) {
      this.addPersistentLog(message, type);
    }
  }

  formatTime(date: Date): string {
    return date.toLocaleTimeString('pt-BR', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  }

  getLogClass(type: string): string {
    switch (type) {
      case 'success': return 'log-success';
      case 'error': return 'log-error';
      case 'progress': return 'log-progress';
      default: return 'log-info';
    }
  }

  // Results system methods
  async selectJsonFile(): Promise<void> {
    try {
      const selected = await open({
        multiple: false,
        defaultPath: this.outputDirectory,
        filters: [
          {
            name: 'JSON Files',
            extensions: ['json']
          }
        ]
      });
      
      if (selected && typeof selected === 'string') {
        // Obter informações do arquivo selecionado
        const fileInfo = await invoke<JsonFileInfo>('get_json_file_info', {
          file_path: selected
        });
        this.selectedJsonFile = fileInfo;
      }
    } catch (error) {
      console.error('Error selecting JSON file:', error);
      this.addLog('❌ Erro ao selecionar arquivo JSON', 'error');
    }
  }

  async loadJsonFiles(): Promise<void> {
    if (!this.outputDirectory) {
      this.addLog('❌ Pasta de resultados não definida', 'error');
      return;
    }

    this.loadingJsonFiles = true;
    this.availableJsonFiles = [];

    try {
      this.addLog(`🔍 Verificando pasta de resultados: ${this.outputDirectory}`, 'info');
      
      // Verificar se o diretório existe e criar exemplo se necessário
      const verifyResult = await invoke<string>('verify_output_directory');
      this.addLog(`✅ ${verifyResult}`, 'success');
      
      const jsonFiles = await invoke<string[]>('list_json_files', {
        directory: this.outputDirectory
      });

      this.addLog(`📁 Encontrados ${jsonFiles.length} arquivos JSON`, 'info');

      if (jsonFiles.length === 0) {
        this.addLog('ℹ️ Nenhum arquivo JSON encontrado na pasta de resultados', 'info');
        this.addLog('💡 Execute o processamento primeiro para gerar arquivos JSON', 'info');
        return;
      }

      // Obter informações de cada arquivo
      const fileInfoPromises = jsonFiles.map(async (filePath) => {
        try {
          const fileInfo = await invoke<JsonFileInfo>('get_json_file_info', { file_path: filePath });
          
          if (fileInfo.error) {
            this.addLog(`⚠️ Arquivo com problema: ${filePath} - ${fileInfo.error}`, 'error');
          } else {
            this.addLog(`✅ Arquivo carregado: ${fileInfo.file_name}`, 'info');
          }
          
          return fileInfo;
        } catch (error) {
          this.addLog(`❌ Erro crítico ao obter informações do arquivo: ${filePath}`, 'error');
          console.error('Error getting file info:', error);
          
          // Retornar um objeto de erro personalizado
          return {
            file_name: filePath.split(/[/\\]/).pop() || 'arquivo_desconhecido',
            file_path: filePath,
            file_size: 0,
            modified_timestamp: 0,
            error: `Erro crítico: ${error}`
          } as JsonFileInfo;
        }
      });

      const fileInfos = await Promise.all(fileInfoPromises);
      this.availableJsonFiles = fileInfos;
      
      const validFiles = fileInfos.filter(info => !info.error).length;
      const errorFiles = fileInfos.filter(info => info.error).length;
      
      this.addLog(`✅ ${validFiles} arquivos JSON válidos carregados`, 'success');
      if (errorFiles > 0) {
        this.addLog(`⚠️ ${errorFiles} arquivos com problemas detectados`, 'error');
      }
    } catch (error) {
      console.error('Error loading JSON files:', error);
      this.addLog(`❌ Erro ao carregar arquivos JSON: ${error}`, 'error');
      this.addLog(`📁 Pasta verificada: ${this.outputDirectory}`, 'error');
    } finally {
      this.loadingJsonFiles = false;
    }
  }

  async loadJsonData(): Promise<void> {
    if (!this.selectedJsonFile) {
      this.addLog('❌ Nenhum arquivo JSON selecionado', 'error');
      return;
    }

    this.loadingJsonData = true;
    this.jsonData = null;

    try {
      const data = await invoke<JsonData>('read_json_file', {
        file_path: this.selectedJsonFile.file_path
      });

      this.jsonData = data;
      this.addLog(`📄 Dados carregados: ${data.propostas.length} propostas`, 'success');
    } catch (error) {
      console.error('Error loading JSON data:', error);
      this.addLog('❌ Erro ao carregar dados JSON', 'error');
    } finally {
      this.loadingJsonData = false;
    }
  }

  selectJsonFromList(fileInfo: JsonFileInfo): void {
    this.selectedJsonFile = fileInfo;
    this.addLog(`📁 Arquivo selecionado: ${fileInfo.file_name}`, 'info');
  }

  formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString('pt-BR');
  }

  async onVerboseChange(newValue?: boolean): Promise<void> {
    if (newValue !== undefined) {
      this.verbose = newValue;
    }
    await this.updateVerboseConfig();
  }

  async openPdfFolder(): Promise<void> {
    try {
      // Usar API do Tauri para abrir a pasta no explorador do sistema
      await invoke('open_pdf_file', {
        file_path: this.pdfDirectory
      });
      
      this.addLog(`📁 Pasta PDF aberta: ${this.pdfDirectory}`, 'info');
    } catch (error) {
      console.error('Error opening PDF folder:', error);
      this.addLog(`❌ Erro ao abrir pasta PDF: ${error}`, 'error');
    }
  }

  async openOutputFolder(): Promise<void> {
    try {
      await invoke<boolean>('open_folder', {
        path: this.outputDirectory
      });
      this.addLog('📁 Pasta de resultados aberta no explorador', 'success');
    } catch (error) {
      console.error('Error opening output folder:', error);
      this.addLog('❌ Erro ao abrir pasta de resultados', 'error');
    }
  }

  async processFixedDirectory(): Promise<void> {
    this.processing = true;
    this.processingResult = null;
    this.progressLogs = [];
    this.totalProposals = 0;
    this.currentSessionId = Date.now().toString();

    this.addLog('🔄 Iniciando processamento da pasta PDF...', 'info');

    try {
      const result = await invoke<ProcessingResult>('process_pdf_fixed_directory', {
        verbose: this.verbose,
        session_id: this.currentSessionId
      });

      if (result.session_id) {
        this.currentSessionId = result.session_id;
        this.startProgressMonitoring(result.session_id);
      }

      this.processingResult = result;
      this.totalProposals = result.propostas.length;
      
      if (result.success) {
        this.addLog(`✅ Pasta PDF processada com sucesso! ${result.total_processed} arquivos processados`, 'success');
        this.addLog(`📊 Total de propostas: ${result.propostas.length}`, 'success');
        if (result.json_file_path) {
          this.addLog(`📄 Arquivo JSON salvo em: ${result.json_file_path}`, 'success');
        }
      } else {
        this.addLog(`❌ Erro no processamento: ${result.message}`, 'error');
      }
    } catch (error) {
      console.error('Error processing fixed directory:', error);
      this.addLog('❌ Erro durante o processamento da pasta PDF', 'error');
      this.processingResult = {
        success: false,
        message: `Erro durante o processamento: ${error}`,
        propostas: [],
        total_processed: 0
      };
    } finally {
      this.processing = false;
      this.stopProgressMonitoring();
    }
  }

  async onTabChange(tab: 'processing' | 'logs' | 'results'): Promise<void> {
    this.activeTab = tab;
    
    if (tab === 'results') {
      // Carregar automaticamente a lista de arquivos JSON quando a aba de resultados for selecionada
      await this.loadJsonFiles();
    }
  }

  // NEW METHODS FOR PDF NAVIGATOR
  async loadPdfFilesList(): Promise<void> {
    if (!this.pdfDirectory) {
      this.addLog('❌ Pasta PDF não definida', 'error');
      return;
    }

    this.loadingPdfFiles = true;
    this.availablePdfFiles = [];

    try {
      this.addLog(`🔍 Carregando arquivos PDF da pasta: ${this.pdfDirectory}`, 'info');
      
      const pdfFiles = await invoke<string[]>('list_pdf_files', {
        directory: this.pdfDirectory
      });

      this.addLog(`📁 Encontrados ${pdfFiles.length} arquivos PDF`, 'info');

      if (pdfFiles.length === 0) {
        this.addLog('ℹ️ Nenhum arquivo PDF encontrado na pasta', 'info');
        this.addLog('💡 Adicione arquivos PDF na pasta para processamento', 'info');
        return;
      }

      // Obter informações de cada arquivo
      const fileInfoPromises = pdfFiles.map(async (filePath) => {
        try {
          const fileInfo = await invoke<PdfFileInfo>('get_pdf_file_info', { filePath });
          return fileInfo;
        } catch (error) {
          console.error('Error getting PDF file info:', error);
          
          // Retornar um objeto básico se houver erro
          return {
            file_name: filePath.split(/[/\\]/).pop() || 'arquivo_desconhecido',
            file_path: filePath,
            file_size: 0,
            modified_timestamp: 0
          } as PdfFileInfo;
        }
      });

      const fileInfos = await Promise.all(fileInfoPromises);
      
      // Ordenar por data de modificação (mais recente primeiro)
      this.availablePdfFiles = fileInfos.sort((a, b) => 
        b.modified_timestamp - a.modified_timestamp
      );
      
      this.addLog(`✅ ${this.availablePdfFiles.length} arquivos PDF carregados`, 'success');
    } catch (error) {
      console.error('Error loading PDF files:', error);
      this.addLog(`❌ Erro ao carregar arquivos PDF: ${error}`, 'error');
    } finally {
      this.loadingPdfFiles = false;
    }
  }

  selectPdfFile(fileInfo: PdfFileInfo): void {
    this.selectedPdfFile = fileInfo;
    this.addLog(`📄 Arquivo selecionado: ${fileInfo.file_name}`, 'info');
  }

  async viewPdfFile(fileInfo: PdfFileInfo): Promise<void> {
    console.log('🔍 viewPdfFile chamado para:', fileInfo.file_name);
    try {
      this.addLog(`👁️ Abrindo visualizador para: ${fileInfo.file_name}`, 'info');
      
      // Usar API do Tauri para abrir o arquivo no visualizador padrão do sistema
      await invoke('open_pdf_file', {
        filePath: fileInfo.file_path
      });
      
      console.log('✅ PDF aberto com sucesso:', fileInfo.file_name);
      this.addLog(`✅ PDF aberto no visualizador padrão`, 'success');
    } catch (error) {
      console.error('Error opening PDF file:', error);
      this.addLog(`❌ Erro ao abrir PDF: ${error}`, 'error');
    }
  }

  getFileTooltip(fileInfo: PdfFileInfo): string {
    const sizeFormatted = this.formatFileSize(fileInfo.file_size);
    const dateFormatted = this.formatDate(fileInfo.modified_timestamp);
    return `${fileInfo.file_name}\nTamanho: ${sizeFormatted}\nModificado: ${dateFormatted}\n\nClique para selecionar\nDuplo clique para abrir`;
  }

  async processSinglePdfFile(fileInfo: PdfFileInfo): Promise<void> {
    if (!fileInfo || !this.outputDirectory) {
      this.addLog('❌ Arquivo ou pasta de saída não selecionados', 'error');
      return;
    }

    this.processing = true;
    this.processingResult = null;
    this.progressLogs = [];
    this.totalProposals = 0;
    this.currentSessionId = Date.now().toString();

    this.addLog(`🔄 Processando arquivo: ${fileInfo.file_name}`, 'info');

    try {
      const result = await invoke<ProcessingResult>('process_pdf_file', {
        file_path: fileInfo.file_path,
        output_dir: this.outputDirectory,
        verbose: this.verbose
      });

      if (result.session_id) {
        this.currentSessionId = result.session_id;
        this.startProgressMonitoring(result.session_id);
      }

      this.processingResult = result;
      this.totalProposals = result.propostas.length;
      
      if (result.success) {
        this.addLog(`✅ Arquivo processado com sucesso! ${result.propostas.length} propostas encontradas`, 'success');
        if (result.json_file_path) {
          this.addLog(`📄 Arquivo JSON salvo em: ${result.json_file_path}`, 'success');
        }
      } else {
        this.addLog(`❌ Erro no processamento: ${result.message}`, 'error');
      }
    } catch (error) {
      console.error('Error processing file:', error);
      this.addLog('❌ Erro durante o processamento', 'error');
      this.processingResult = {
        success: false,
        message: `Erro durante o processamento: ${error}`,
        propostas: [],
        total_processed: 0
      };
    } finally {
      this.processing = false;
      this.stopProgressMonitoring();
    }
  }

  // NEW METHODS FOR DIRECTORY MANAGEMENT
  async changePdfDirectory(): Promise<void> {
    try {
      this.isChangingDirectories = true;
      
      const selected = await open({
        directory: true,
        defaultPath: this.pdfDirectory
      });
      
      if (selected && typeof selected === 'string') {
        this.pdfDirectory = selected;
        this.addLog(`📁 Nova pasta PDF selecionada: ${selected}`, 'info');
        
        // Atualizar configuração
        await this.updateDirectoriesConfig();
        
        // Recarregar lista de arquivos PDF
        await this.loadPdfFilesList();
      }
    } catch (error) {
      console.error('Error changing PDF directory:', error);
      this.addLog('❌ Erro ao alterar pasta PDF', 'error');
    } finally {
      this.isChangingDirectories = false;
    }
  }

  async changeOutputDirectory(): Promise<void> {
    try {
      this.isChangingDirectories = true;
      
      const selected = await open({
        directory: true,
        defaultPath: this.outputDirectory
      });
      
      if (selected && typeof selected === 'string') {
        this.outputDirectory = selected;
        this.addLog(`📤 Nova pasta de saída selecionada: ${selected}`, 'info');
        
        // Atualizar configuração
        await this.updateDirectoriesConfig();
      }
    } catch (error) {
      console.error('Error changing output directory:', error);
      this.addLog('❌ Erro ao alterar pasta de saída', 'error');
    } finally {
      this.isChangingDirectories = false;
    }
  }

  async resetToDefaultDirectories(): Promise<void> {
    try {
      this.isChangingDirectories = true;
      
      // Obter pastas padrão
      const defaultPdfDir = await invoke<string>('get_pdf_directory');
      const defaultOutputDir = await invoke<string>('get_output_directory');
      
      this.pdfDirectory = defaultPdfDir;
      this.outputDirectory = defaultOutputDir;
      
      // Atualizar configuração
      await this.updateDirectoriesConfig();
      
      // Recarregar lista de arquivos PDF
      await this.loadPdfFilesList();
      
      this.addLog('🔄 Pastas restauradas para padrão', 'success');
      this.addLog(`  • PDF: ${this.pdfDirectory}`, 'info');
      this.addLog(`  • Resultados: ${this.outputDirectory}`, 'info');
    } catch (error) {
      console.error('Error resetting directories:', error);
      this.addLog('❌ Erro ao restaurar pastas padrão', 'error');
    } finally {
      this.isChangingDirectories = false;
    }
  }

  async debugAndRepairConfig(): Promise<void> {
    try {
      this.addLog('🔧 Executando debug e reparo da configuração...', 'info');
      
      const result = await invoke<ConfigResult>('debug_and_repair_config');
      
      if (result.success) {
        this.addLog('✅ Debug e reparo executados com sucesso', 'success');
        
        // Log the detailed information
        const lines = result.message.split('\n');
        for (const line of lines) {
          if (line.trim()) {
            this.addLog(`  ${line.trim()}`, 'info');
          }
        }
        
        // Reload configuration after repair
        await this.loadConfiguration();
      } else {
        this.addLog('❌ Falha no debug e reparo', 'error');
      }
    } catch (error) {
      console.error('Error debugging and repairing config:', error);
      this.addLog('❌ Erro crítico no debug e reparo', 'error');
    }
  }

  async showAppDirectoriesInfo(): Promise<void> {
    try {
      this.addLog('🔍 Obtendo informações dos diretórios...', 'info');
      
      const dirInfo = await invoke<any>('get_app_directories_info');
      
      this.addLog('📂 Informações detalhadas dos diretórios:', 'info');
      this.addLog(`  • Home do usuário: ${dirInfo.home_directory}`, 'info');
      this.addLog(`  • Diretório de config: ${dirInfo.config_directory}`, 'info');
      this.addLog(`  • PDF padrão: ${dirInfo.default_pdf_directory}`, 'info');
      this.addLog(`  • Saída padrão: ${dirInfo.default_output_directory}`, 'info');
      this.addLog(`  • Arquivo de config: ${dirInfo.config_file_path}`, 'info');
      this.addLog(`  • Config existe: ${dirInfo.config_file_exists ? '✅' : '❌'}`, dirInfo.config_file_exists ? 'success' : 'error');
      this.addLog(`  • Pasta PDF existe: ${dirInfo.pdf_directory_exists ? '✅' : '❌'}`, dirInfo.pdf_directory_exists ? 'success' : 'error');
      this.addLog(`  • Pasta output existe: ${dirInfo.output_directory_exists ? '✅' : '❌'}`, dirInfo.output_directory_exists ? 'success' : 'error');
      
    } catch (error) {
      console.error('Error getting app directories info:', error);
      this.addLog('❌ Erro ao obter informações dos diretórios', 'error');
    }
  }

  // NEW: Integração com componentes filhos
  onPdfDirectoryChanged(newPath: string): void {
    this.pdfDirectory = newPath;
    this.updateDirectoriesConfig();
    this.addLog(`📁 Pasta PDF atualizada: ${newPath}`, 'success');
  }

  onOutputDirectoryChanged(newPath: string): void {
    this.outputDirectory = newPath;
    this.updateDirectoriesConfig();
    this.addLog(`📤 Pasta de resultados atualizada: ${newPath}`, 'success');
  }

  onPdfFileSelected(fileInfo: PdfFileInfo): void {
    this.selectedPdfFile = fileInfo;
    this.addLog(`📄 Arquivo selecionado: ${fileInfo.file_name}`, 'info');
  }

  onProcessPdfFile(fileInfo: PdfFileInfo): void {
    this.processSinglePdfFile(fileInfo);
  }

  onStatusMessage(event: {message: string, type: 'success' | 'error' | 'info'}): void {
    this.addLog(event.message, event.type);
  }
}