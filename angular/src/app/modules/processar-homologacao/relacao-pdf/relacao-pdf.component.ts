import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';

declare global {
  interface Window {
    __TAURI__: any;
  }
}

interface PdfFileInfo {
  file_name: string;
  file_path: string;
  file_size: number;
  modified_timestamp: number;
}

interface ProcessingResult {
  success: boolean;
  message: string;
  propostas: any[];
  total_processed: number;
  json_file_path?: string;
  session_id?: string;
}

@Component({
  selector: 'app-relacao-pdf',
  imports: [CommonModule, FormsModule],
  templateUrl: './relacao-pdf.component.html',
  styleUrl: './relacao-pdf.component.css'
})
export class RelacaoPdfComponent implements OnInit {
  @Input() pdfDirectory: string = '';
  @Input() outputDirectory: string = '';
  @Input() processing: boolean = false;
  @Output() fileSelected = new EventEmitter<PdfFileInfo>();
  @Output() processFile = new EventEmitter<PdfFileInfo>();
  @Output() statusMessage = new EventEmitter<{message: string, type: 'success' | 'error' | 'info'}>();

  availablePdfFiles: PdfFileInfo[] = [];
  selectedPdfFile: PdfFileInfo | null = null;
  loadingPdfFiles: boolean = false;
  searchTerm: string = '';
  sortBy: 'name' | 'date' | 'size' = 'name';
  sortDirection: 'asc' | 'desc' = 'asc';
  selectedFiles: Set<string> = new Set();

  ngOnInit(): void {
    this.loadPdfFilesList();
  }

  async loadPdfFilesList(): Promise<void> {
    if (!this.pdfDirectory) {
      this.availablePdfFiles = [];
      return;
    }

    this.loadingPdfFiles = true;
    this.availablePdfFiles = [];

    try {
      const fileInfos = await window.__TAURI__.invoke('get_pdf_files_info', {
        directory: this.pdfDirectory
      });

      this.availablePdfFiles = fileInfos || [];
      this.sortFiles();
      
      this.statusMessage.emit({
        message: `${this.availablePdfFiles.length} arquivo(s) PDF encontrado(s)`,
        type: 'info'
      });
    } catch (error: any) {
      console.error('Erro ao carregar lista de PDFs:', error);
      this.statusMessage.emit({
        message: `Erro ao carregar PDFs: ${error.message || error}`,
        type: 'error'
      });
    } finally {
      this.loadingPdfFiles = false;
    }
  }

  selectPdfFile(fileInfo: PdfFileInfo): void {
    this.selectedPdfFile = fileInfo;
    this.fileSelected.emit(fileInfo);
  }

  async viewPdfFile(fileInfo: PdfFileInfo): Promise<void> {
    try {
      await window.__TAURI__.invoke('open_pdf_file', { 
        filePath: fileInfo.file_path 
      });
    } catch (error: any) {
      console.error('Erro ao abrir PDF:', error);
      this.statusMessage.emit({
        message: `Erro ao abrir PDF: ${error.message || error}`,
        type: 'error'
      });
    }
  }

  async processSinglePdfFile(fileInfo: PdfFileInfo): Promise<void> {
    if (!fileInfo || !this.outputDirectory) {
      this.statusMessage.emit({
        message: 'Pasta de saída não configurada',
        type: 'error'
      });
      return;
    }

    this.processFile.emit(fileInfo);
  }

  toggleFileSelection(fileInfo: PdfFileInfo): void {
    if (this.selectedFiles.has(fileInfo.file_path)) {
      this.selectedFiles.delete(fileInfo.file_path);
    } else {
      this.selectedFiles.add(fileInfo.file_path);
    }
  }

  isFileSelected(fileInfo: PdfFileInfo): boolean {
    return this.selectedFiles.has(fileInfo.file_path);
  }

  selectAllFiles(): void {
    if (this.selectedFiles.size === this.getFilteredFiles().length) {
      this.selectedFiles.clear();
    } else {
      this.getFilteredFiles().forEach(file => {
        this.selectedFiles.add(file.file_path);
      });
    }
  }

  getSelectedFilesCount(): number {
    return this.selectedFiles.size;
  }

  async processSelectedFiles(): Promise<void> {
    if (this.selectedFiles.size === 0) {
      this.statusMessage.emit({
        message: 'Nenhum arquivo selecionado',
        type: 'error'
      });
      return;
    }

    // Processar arquivos selecionados em lote
    // Esta funcionalidade pode ser implementada no futuro
    this.statusMessage.emit({
      message: 'Processamento em lote será implementado em breve',
      type: 'info'
    });
  }

  changeSortBy(field: 'name' | 'date' | 'size'): void {
    if (this.sortBy === field) {
      this.sortDirection = this.sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      this.sortBy = field;
      this.sortDirection = 'asc';
    }
    this.sortFiles();
  }

  sortFiles(): void {
    this.availablePdfFiles.sort((a, b) => {
      let comparison = 0;
      
      switch (this.sortBy) {
        case 'name':
          comparison = a.file_name.localeCompare(b.file_name);
          break;
        case 'date':
          comparison = a.modified_timestamp - b.modified_timestamp;
          break;
        case 'size':
          comparison = a.file_size - b.file_size;
          break;
      }

      return this.sortDirection === 'desc' ? -comparison : comparison;
    });
  }

  getFilteredFiles(): PdfFileInfo[] {
    if (!this.searchTerm) {
      return this.availablePdfFiles;
    }
    
    const searchLower = this.searchTerm.toLowerCase();
    return this.availablePdfFiles.filter(file => 
      file.file_name.toLowerCase().includes(searchLower)
    );
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

  getFileTooltip(fileInfo: PdfFileInfo): string {
    const sizeFormatted = this.formatFileSize(fileInfo.file_size);
    const dateFormatted = this.formatDate(fileInfo.modified_timestamp);
    return `${fileInfo.file_name}\nTamanho: ${sizeFormatted}\nModificado: ${dateFormatted}\n\nClique para selecionar\nDuplo clique para abrir`;
  }

  async refreshFilesList(): Promise<void> {
    this.selectedFiles.clear();
    this.selectedPdfFile = null;
    await this.loadPdfFilesList();
  }

  trackByPath(index: number, file: PdfFileInfo): string {
    return file.file_path;
  }
}
