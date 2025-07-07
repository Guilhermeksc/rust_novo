import { Component, OnInit, Input, ChangeDetectorRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationService, HeaderButton } from '../../services/navigation.service';
import { ButtonComponent } from '../button/button.component';

@Component({
  selector: 'app-header-content',
  standalone: true,
  imports: [CommonModule, ButtonComponent],
  templateUrl: './header-content.component.html',
  styleUrl: './header-content.component.css'
})
export class HeaderContentComponent implements OnInit {
  @Input() title: string = '';
  @Input() subtitle: string = '';
  @Input() showBreadcrumbs: boolean = true;
  @Input() showUserInfo: boolean = true;
  
  currentPage: string = '';
  currentTime: string = '';
  headerButtons: HeaderButton[] = [];
  
  constructor(
    private navigationService: NavigationService,
    private cdr: ChangeDetectorRef
  ) {}

  ngOnInit(): void {
    this.updateTime();
    this.setupTimeUpdate();
    
    // Observar mudanças de página
    this.navigationService.currentPage$.subscribe(page => {
      this.currentPage = page;
      this.headerButtons = this.navigationService.getHeaderButtons(page);
      // Forçar detecção de mudanças
      this.cdr.detectChanges();
    });
  }

  private updateTime(): void {
    const now = new Date();
    this.currentTime = now.toLocaleString('pt-BR', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  private setupTimeUpdate(): void {
    setInterval(() => {
      this.updateTime();
    }, 60000); // Atualizar a cada minuto
  }

  getPageTitle(): string {
    // Se um título específico foi passado via @Input e tem conteúdo, use-o
    if (this.title && this.title.trim()) return this.title;
    
    // Caso contrário, sempre use o título da página atual
    const menuItem = this.navigationService.getMenuItem(this.currentPage);
    return menuItem ? menuItem.label : 'Licitação 360';
  }

  getPageSubtitle(): string {
    // Se um subtítulo específico foi passado via @Input e tem conteúdo, use-o
    if (this.subtitle && this.subtitle.trim()) return this.subtitle;
    
    // Caso contrário, sempre use a descrição da página atual
    const menuItem = this.navigationService.getMenuItem(this.currentPage);
    return menuItem && menuItem.description ? menuItem.description : 'Sistema de processamento de licitações';
  }

  getPageIcon(): string {
    const menuItem = this.navigationService.getMenuItem(this.currentPage);
    return menuItem ? menuItem.icon : '📋';
  }

  // Método para lidar com cliques nos botões dinâmicos
  onButtonClick(button: HeaderButton): void {
    this.navigationService.emitButtonAction(button.action);
  }

  // Métodos dos botões padrão (mantidos para compatibilidade)
  onRefresh(): void {
    window.location.reload();
  }

  onExport(): void {
    // Implementar lógica de exportação
    console.log('Exportar dados da página atual');
  }

  onHelp(): void {
    // Implementar lógica de ajuda
    console.log('Mostrar ajuda para a página atual');
  }
}
