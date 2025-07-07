import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

interface ConfigResult {
  success: boolean;
  message: string;
  config?: any;
}

@Component({
  selector: 'app-selecionar-pasta',
  imports: [CommonModule, FormsModule],
  templateUrl: './selecionar-pasta.component.html',
  styleUrl: './selecionar-pasta.component.css'
})
export class SelecionarPastaComponent {
  @Input() label: string = 'Selecionar Pasta';
  @Input() currentPath: string = '';
  @Input() pastaType: 'pdf' | 'output' = 'pdf';
  @Input() disabled: boolean = false;
  @Output() pathChanged = new EventEmitter<string>();
  @Output() statusMessage = new EventEmitter<{message: string, type: 'success' | 'error' | 'info'}>();

  isSelecting: boolean = false;

  async selecionarPasta(): Promise<void> {
    if (this.disabled || this.isSelecting) return;

    this.isSelecting = true;
    
    try {
      const title = this.pastaType === 'pdf' ? 'Selecionar Pasta de PDFs' : 'Selecionar Pasta de Resultados';
      
      // Usar o dialog plugin do Tauri
      const selectedPath = await open({
        title,
        directory: true,
        multiple: false,
        defaultPath: this.currentPath || undefined
      });

      if (selectedPath && typeof selectedPath === 'string') {
        // Emitir o caminho primeiro para que o componente pai possa atualizar
        this.pathChanged.emit(selectedPath);
        
        // Atualizar a configuração usando o comando específico
        try {
          const updateCommand = this.pastaType === 'pdf' ? 'update_pdf_directory' : 'update_output_directory';
          const result = await invoke<ConfigResult>(updateCommand, {
            path: selectedPath
          });

          if (result.success) {
            this.statusMessage.emit({
              message: `Pasta ${this.pastaType === 'pdf' ? 'PDF' : 'de resultados'} salva com sucesso`,
              type: 'success'
            });
          } else {
            this.statusMessage.emit({
              message: `Erro ao salvar configuração: ${result.message}`,
              type: 'error'
            });
          }
        } catch (configError) {
          console.warn('Erro ao salvar configuração da pasta, mas pasta foi selecionada:', configError);
          this.statusMessage.emit({
            message: 'Pasta selecionada, mas houve problema ao salvar a configuração',
            type: 'info'
          });
        }
      } else {
        this.statusMessage.emit({
          message: 'Seleção de pasta cancelada',
          type: 'info'
        });
      }
    } catch (error: any) {
      console.error('Erro ao selecionar pasta:', error);
      this.statusMessage.emit({
        message: `Erro ao selecionar pasta: ${error.message || error}`,
        type: 'error'
      });
    } finally {
      this.isSelecting = false;
    }
  }

  async abrirPastaAtual(): Promise<void> {
    if (!this.currentPath) return;

    try {
      await invoke<boolean>('open_folder', { path: this.currentPath });
      this.statusMessage.emit({
        message: 'Pasta aberta no explorador',
        type: 'success'
      });
    } catch (error: any) {
      console.error('Erro ao abrir pasta:', error);
      this.statusMessage.emit({
        message: `Erro ao abrir pasta: ${error.message || error}`,
        type: 'error'
      });
    }
  }

  getTruncatedPath(): string {
    if (!this.currentPath) return 'Nenhuma pasta selecionada';
    
    const maxLength = 50;
    if (this.currentPath.length <= maxLength) {
      return this.currentPath;
    }
    
    // Truncar no meio mantendo início e fim
    const start = this.currentPath.substring(0, 20);
    const end = this.currentPath.substring(this.currentPath.length - 25);
    return `${start}...${end}`;
  }

  getPathExists(): boolean {
    return !!(this.currentPath && this.currentPath.length > 0);
  }
}
