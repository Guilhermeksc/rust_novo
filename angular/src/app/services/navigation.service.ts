import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';

export interface HeaderButton {
  id: string;
  label: string;
  icon: string;
  tooltip?: string;
  action: string;
  style?: 'primary' | 'secondary' | 'accent' | 'danger';
}

export interface MenuItem {
  id: string;
  label: string;
  icon: string;
  description?: string;
  headerButtons?: HeaderButton[];
}

@Injectable({
  providedIn: 'root'
})
export class NavigationService {
  private currentPageSubject = new BehaviorSubject<string>('processar-homologacao');
  public currentPage$ = this.currentPageSubject.asObservable();

  private sidebarCollapsedSubject = new BehaviorSubject<boolean>(false);
  public sidebarCollapsed$ = this.sidebarCollapsedSubject.asObservable();

  private sidebarOpenSubject = new BehaviorSubject<boolean>(false);
  public sidebarOpen$ = this.sidebarOpenSubject.asObservable();

  // Subject para comunicar ações dos botões
  private buttonActionSubject = new BehaviorSubject<string>('');
  public buttonAction$ = this.buttonActionSubject.asObservable();

  public menuItems: MenuItem[] = [
    {
      id: 'termo-referencia',
      label: 'Termo de Referência',
      icon: '📄',
      description: 'Gestão de termos de referência',
      headerButtons: [
        {
          id: 'importar-tabela',
          label: 'Importar',
          icon: '📤',
          tooltip: 'Importar nova tabela Excel, ODF ou CSV',
          action: 'carregarTabela',
          style: 'secondary'
        },
        {
          id: 'abrir-tabela-nova',
          label: 'Abrir Tabela Nova',
          icon: 'assets/icons/excel.png',
          tooltip: 'Cria arquivo Excel modelo em branco para preenchimento',
          action: 'abrirTabelaNova',
          style: 'primary'
        }
      ]
    },
    {
      id: 'processar-homologacao',
      label: 'Processar Termos de Homologação',
      icon: '⚡',
      description: 'Processamento de PDFs de homologação',
      headerButtons: [
        {
          id: 'processar-pdf',
          label: 'Processar PDF',
          icon: '📄',
          tooltip: 'Processar arquivo PDF',
          action: 'processarPdf',
          style: 'primary'
        },
        {
          id: 'ver-resultados',
          label: 'Ver Resultados',
          icon: '📊',
          tooltip: 'Ver resultados do processamento',
          action: 'verResultados',
          style: 'secondary'
        }
      ]
    },
    {
      id: 'dados-empresas',
      label: 'Dados das Empresas',
      icon: '🏢',
      description: 'Cadastro e gestão de empresas',
      headerButtons: [
        {
          id: 'nova-empresa',
          label: 'Nova Empresa',
          icon: '➕',
          tooltip: 'Cadastrar nova empresa',
          action: 'novaEmpresa',
          style: 'primary'
        },
        {
          id: 'importar-dados',
          label: 'Importar',
          icon: '📤',
          tooltip: 'Importar dados de empresas',
          action: 'importarDados',
          style: 'secondary'
        }
      ]
    },
    {
      id: 'gerar-ata',
      label: 'Gerar Ata',
      icon: '📝',
      description: 'Geração de atas de processos',
      headerButtons: [
        {
          id: 'nova-ata',
          label: 'Nova Ata',
          icon: '📝',
          tooltip: 'Criar nova ata',
          action: 'novaAta',
          style: 'primary'
        },
        {
          id: 'modelos-ata',
          label: 'Modelos',
          icon: '📋',
          tooltip: 'Gerenciar modelos de ata',
          action: 'modelosAta',
          style: 'secondary'
        }
      ]
    },
    {
      id: 'indices-economicidade',
      label: 'Índices de Economicidade',
      icon: '📊',
      description: 'Análise de economicidade',
      headerButtons: [
        {
          id: 'calcular-indices',
          label: 'Calcular',
          icon: '🧮',
          tooltip: 'Calcular índices de economicidade',
          action: 'calcularIndices',
          style: 'primary'
        },
        {
          id: 'exportar-relatorio',
          label: 'Exportar',
          icon: '📤',
          tooltip: 'Exportar relatório',
          action: 'exportarRelatorio',
          style: 'secondary'
        }
      ]
    },
    {
      id: 'configuracoes',
      label: 'Configurações',
      icon: '⚙️',
      description: 'Configurações do sistema',
      headerButtons: [
        {
          id: 'salvar-config',
          label: 'Salvar',
          icon: '💾',
          tooltip: 'Salvar configurações',
          action: 'salvarConfig',
          style: 'primary'
        },
        {
          id: 'restaurar-config',
          label: 'Restaurar',
          icon: '🔄',
          tooltip: 'Restaurar configurações padrão',
          action: 'restaurarConfig',
          style: 'secondary'
        }
      ]
    }
  ];

  constructor() { }

  getCurrentPage(): string {
    return this.currentPageSubject.value;
  }

  setCurrentPage(pageId: string): void {
    if (this.menuItems.some(item => item.id === pageId)) {
      this.currentPageSubject.next(pageId);
    }
  }

  getMenuItem(pageId: string): MenuItem | undefined {
    return this.menuItems.find(item => item.id === pageId);
  }

  getHeaderButtons(pageId: string): HeaderButton[] {
    const menuItem = this.getMenuItem(pageId);
    return menuItem?.headerButtons || [];
  }

  // Método para emitir ações dos botões
  emitButtonAction(action: string): void {
    this.buttonActionSubject.next(action);
  }

  getSidebarCollapsed(): boolean {
    return this.sidebarCollapsedSubject.value;
  }

  setSidebarCollapsed(collapsed: boolean): void {
    this.sidebarCollapsedSubject.next(collapsed);
  }

  toggleSidebar(): void {
    this.setSidebarCollapsed(!this.getSidebarCollapsed());
  }

  getSidebarOpen(): boolean {
    return this.sidebarOpenSubject.value;
  }

  setSidebarOpen(open: boolean): void {
    this.sidebarOpenSubject.next(open);
  }

  toggleSidebarOpen(): void {
    this.setSidebarOpen(!this.getSidebarOpen());
  }
}
