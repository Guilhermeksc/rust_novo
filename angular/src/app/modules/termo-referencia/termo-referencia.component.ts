import { Component, OnInit, ViewChild, ElementRef, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import * as XLSX from 'xlsx';
import { NavigationService } from '../../services/navigation.service';
import { Subscription } from 'rxjs';

// Interface para o termo de referência
interface TermoReferencia {
  item: string;
  catalogo: string;
  descricao: string;
  descricao_detalhada: string;
}

// Interface para histórico de arquivos importados
interface ArquivoImportado {
  id: string;
  nome: string;
  nomeOriginal: string;
  dataImportacao: Date;
  quantidadeItens: number;
  dados: TermoReferencia[];
}

@Component({
    selector: 'app-termo-referencia',
    standalone: true,
    imports: [CommonModule, FormsModule],
    templateUrl: './termo-referencia.component.html',
    styleUrls: ['./termo-referencia.component.css']
})
export class TermoReferenciaComponent implements OnInit, OnDestroy {
  @ViewChild('fileInput') fileInput!: ElementRef<HTMLInputElement>;
  
  // Subscriptions para cleanup
  private buttonActionSubscription: Subscription | null = null;
  
  // Propriedades para gerenciar dados
  termosReferencia: TermoReferencia[] = [];
  selectedIndex: number = -1;
  totalItems: number = 0;
  
  // Histórico de arquivos importados
  arquivosImportados: ArquivoImportado[] = [];
  arquivoAtualId: string = '';
  
  // Estados do componente
  loading: boolean = false;
  loadingMessage: string = '';
  editingItem: boolean = false;
  
  // Controle de nome personalizado
  mostrarDialogoNome: boolean = false;
  nomeArquivoPersonalizado: string = '';
  arquivoSelecionadoParaImportacao: File | null = null;
  
  // Item sendo editado
  itemEdicao: TermoReferencia = {
    item: '',
    catalogo: '',
    descricao: '',
    descricao_detalhada: ''
  };
  
  // Configurações
  private readonly COLUNAS_OBRIGATORIAS = ['item', 'catalogo', 'descricao', 'descricao_detalhada'];

  constructor(private navigationService: NavigationService) {}

  ngOnInit(): void {
    this.carregarDadosLocais();
    this.carregarHistoricoArquivos();
    
    // Escutar ações dos botões do header
    this.buttonActionSubscription = this.navigationService.buttonAction$.subscribe(action => {
      if (action) {
        this.handleHeaderButtonAction(action);
      }
    });
  }

  ngOnDestroy(): void {
    // Limpar subscriptions
    if (this.buttonActionSubscription) {
      this.buttonActionSubscription.unsubscribe();
    }
  }

  /**
   * Lida com ações dos botões do header
   */
  private handleHeaderButtonAction(action: string): void {
    switch (action) {
      case 'carregarTabela':
        this.carregarTabela();
        break;
      case 'abrirTabelaNova':
        this.abrirTabelaNova();
        break;
      default:
        // Ação não reconhecida
        break;
    }
  }

  /**
   * Abre uma nova tabela em branco - APENAS CRIA E ABRE ARQUIVO EXCEL
   */
  abrirTabelaNova(): void {
    this.loading = true;
    this.loadingMessage = 'Criando arquivo Excel modelo...';
    
    setTimeout(() => {
      try {
        // Criar estrutura de dados em branco com cabeçalhos
        const dadosEmBranco = [
          {
            item: '',
            catalogo: '',
            descricao: '',
            descricao_detalhada: ''
          }
        ];
        
        // Criar planilha Excel
        const ws = XLSX.utils.json_to_sheet(dadosEmBranco);
        
        // Definir larguras das colunas
        ws['!cols'] = [
          { wch: 10 },  // item
          { wch: 15 },  // catalogo
          { wch: 30 },  // descricao
          { wch: 50 }   // descricao_detalhada
        ];
        
        // Criar workbook e adicionar planilha
        const wb = XLSX.utils.book_new();
        XLSX.utils.book_append_sheet(wb, ws, 'Termo de Referência');
        
        // Gerar nome do arquivo com data
        const dataAtual = new Date().toISOString().split('T')[0];
        const fileName = `termo_referencia_modelo_${dataAtual}.xlsx`;
        
        // Fazer download do arquivo
        XLSX.writeFile(wb, fileName);
        
        this.loading = false;
        alert(`Arquivo modelo criado: ${fileName}\nPreencha os dados e use "Carregar Tabela" para importar!`);
        
      } catch (error) {
        console.error('Erro ao criar arquivo:', error);
        this.loading = false;
        alert('Erro ao criar arquivo Excel.');
      }
    }, 1000);
  }

  /**
   * Carrega/importa uma tabela de arquivo Excel, ODF ou CSV
   */
  carregarTabela(): void {
    this.fileInput.nativeElement.click();
  }

  /**
   * Manipula a seleção de arquivo
   */
  onFileSelected(event: Event): void {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const file = input.files[0];
      
      // Validar formato do arquivo
      const extensao = file.name.split('.').pop()?.toLowerCase();
      if (!['xlsx', 'xls', 'csv', 'ods'].includes(extensao || '')) {
        alert('Formato não suportado. Use: Excel (.xlsx, .xls), OpenDocument (.ods) ou CSV (.csv)');
        return;
      }
      
      // Armazenar arquivo e solicitar nome personalizado
      this.arquivoSelecionadoParaImportacao = file;
      this.nomeArquivoPersonalizado = '';
      this.mostrarDialogoNome = true;
    }
  }

  /**
   * Confirma a importação com nome personalizado
   */
  confirmarImportacao(): void {
    if (!this.nomeArquivoPersonalizado.trim()) {
      alert('Por favor, informe um nome para o arquivo.');
      return;
    }
    
    if (!this.arquivoSelecionadoParaImportacao) {
      alert('Nenhum arquivo selecionado.');
      return;
    }
    
    this.mostrarDialogoNome = false;
    this.processarArquivo(this.arquivoSelecionadoParaImportacao, this.nomeArquivoPersonalizado.trim());
  }

  /**
   * Cancela a importação
   */
  cancelarImportacao(): void {
    this.mostrarDialogoNome = false;
    this.arquivoSelecionadoParaImportacao = null;
    this.nomeArquivoPersonalizado = '';
  }

  /**
   * Processa o arquivo selecionado
   */
  private processarArquivo(file: File, nomePersonalizado: string): void {
    this.loading = true;
    this.loadingMessage = `Processando arquivo: ${file.name}`;
    
    const reader = new FileReader();
    
    reader.onload = (e) => {
      try {
        const data = new Uint8Array(e.target?.result as ArrayBuffer);
        const workbook = XLSX.read(data, { type: 'array' });
        
        // Pega a primeira planilha
        const sheetName = workbook.SheetNames[0];
        const worksheet = workbook.Sheets[sheetName];
        
        // Converte para JSON
        const jsonData = XLSX.utils.sheet_to_json(worksheet, { header: 1 });
        
        this.processarDadosImportados(jsonData, nomePersonalizado, file.name);
      } catch (error) {
        console.error('Erro ao processar arquivo:', error);
        this.loading = false;
        alert('Erro ao processar o arquivo. Verifique se o formato está correto.');
      }
    };
    
    reader.onerror = () => {
      this.loading = false;
      alert('Erro ao ler o arquivo.');
    };
    
    reader.readAsArrayBuffer(file);
  }

  /**
   * Processa os dados importados e valida a estrutura
   */
  private processarDadosImportados(data: any[], nomePersonalizado: string, nomeOriginal: string): void {
    if (data.length < 2) {
      this.loading = false;
      alert('O arquivo deve conter pelo menos uma linha de cabeçalho e uma linha de dados.');
      return;
    }
    
    const cabecalho = data[0].map((col: any) => col.toString().toLowerCase().trim());
    const dadosLinhas = data.slice(1);
    
    // Validar colunas obrigatórias
    const colunasPresentes = this.COLUNAS_OBRIGATORIAS.every(col => 
      cabecalho.includes(col)
    );
    
    if (!colunasPresentes) {
      this.loading = false;
      alert(`O arquivo deve conter as colunas: ${this.COLUNAS_OBRIGATORIAS.join(', ')}`);
      return;
    }
    
    // Mapear índices das colunas
    const indices = {
      item: cabecalho.indexOf('item'),
      catalogo: cabecalho.indexOf('catalogo'),
      descricao: cabecalho.indexOf('descricao'),
      descricao_detalhada: cabecalho.indexOf('descricao_detalhada')
    };
    
    // Converter dados
    const termosImportados: TermoReferencia[] = dadosLinhas
      .filter(linha => linha.length > 0 && linha[indices.item]) // Filtra linhas vazias
      .map(linha => ({
        item: linha[indices.item]?.toString() || '',
        catalogo: linha[indices.catalogo]?.toString() || '',
        descricao: linha[indices.descricao]?.toString() || '',
        descricao_detalhada: linha[indices.descricao_detalhada]?.toString() || ''
      }));
    
    setTimeout(() => {
      // Criar registro do arquivo importado
      const novoArquivo: ArquivoImportado = {
        id: this.gerarId(),
        nome: nomePersonalizado,
        nomeOriginal: nomeOriginal,
        dataImportacao: new Date(),
        quantidadeItens: termosImportados.length,
        dados: termosImportados
      };
      
      // Adicionar ao histórico
      this.adicionarArquivoAoHistorico(novoArquivo);
      
      // Carregar dados do arquivo
      this.carregarArquivoDoHistorico(novoArquivo.id);
      
      this.loading = false;
      alert(`Importação concluída! ${termosImportados.length} itens carregados do arquivo: ${nomePersonalizado}`);
    }, 1000);
  }

  /**
   * Adiciona arquivo ao histórico
   */
  private adicionarArquivoAoHistorico(arquivo: ArquivoImportado): void {
    // Verificar se já existe um arquivo com o mesmo nome personalizado
    const existeArquivo = this.arquivosImportados.find(a => a.nome === arquivo.nome);
    
    if (existeArquivo) {
      if (confirm(`Já existe um arquivo com o nome "${arquivo.nome}". Deseja substituir?`)) {
        // Substituir arquivo existente
        const index = this.arquivosImportados.findIndex(a => a.id === existeArquivo.id);
        this.arquivosImportados[index] = arquivo;
      } else {
        // Adicionar com sufixo
        const timestamp = new Date().toISOString().slice(11, 16);
        arquivo.nome = `${arquivo.nome} (${timestamp})`;
        this.arquivosImportados.unshift(arquivo);
      }
    } else {
      // Adicionar novo arquivo
      this.arquivosImportados.unshift(arquivo); // Adiciona no início
    }
    
    this.salvarHistoricoArquivos();
  }

  /**
   * Carrega arquivo específico do histórico
   */
  carregarArquivoDoHistorico(arquivoId: string): void {
    const arquivo = this.arquivosImportados.find(a => a.id === arquivoId);
    
    if (arquivo) {
      this.termosReferencia = [...arquivo.dados];
      this.totalItems = arquivo.dados.length;
      this.arquivoAtualId = arquivoId;
      this.selectedIndex = -1;
      this.salvarDadosLocais();
    }
  }

  /**
   * Remove arquivo do histórico
   */
  removerArquivoDoHistorico(arquivoId: string): void {
    if (confirm('Tem certeza que deseja remover este arquivo do histórico?')) {
      this.arquivosImportados = this.arquivosImportados.filter(a => a.id !== arquivoId);
      
      // Se o arquivo removido estava sendo usado, limpar dados atuais
      if (this.arquivoAtualId === arquivoId) {
        this.termosReferencia = [];
        this.totalItems = 0;
        this.arquivoAtualId = '';
        this.selectedIndex = -1;
        this.salvarDadosLocais();
      }
      
      this.salvarHistoricoArquivos();
    }
  }

  /**
   * Exporta os dados para Excel
   */
  exportarExcel(): void {
    if (this.termosReferencia.length === 0) {
      alert('Não há dados para exportar.');
      return;
    }
    
    this.loading = true;
    this.loadingMessage = 'Exportando para Excel...';
    
    setTimeout(() => {
      try {
        const ws = XLSX.utils.json_to_sheet(this.termosReferencia);
        
        // Definir larguras das colunas
        ws['!cols'] = [
          { wch: 10 },  // item
          { wch: 15 },  // catalogo
          { wch: 30 },  // descricao
          { wch: 50 }   // descricao_detalhada
        ];
        
        const wb = XLSX.utils.book_new();
        XLSX.utils.book_append_sheet(wb, ws, 'Termos de Referência');
        
        const fileName = `termos_referencia_${new Date().toISOString().split('T')[0]}.xlsx`;
        XLSX.writeFile(wb, fileName);
        
        this.loading = false;
        alert('Arquivo Excel exportado com sucesso!');
      } catch (error) {
        console.error('Erro ao exportar:', error);
        this.loading = false;
        alert('Erro ao exportar arquivo.');
      }
    }, 1000);
  }

  /**
   * Exporta os dados para CSV
   */
  exportarCSV(): void {
    if (this.termosReferencia.length === 0) {
      alert('Não há dados para exportar.');
      return;
    }
    
    this.loading = true;
    this.loadingMessage = 'Exportando para CSV...';
    
    setTimeout(() => {
      try {
        const ws = XLSX.utils.json_to_sheet(this.termosReferencia);
        const csv = XLSX.utils.sheet_to_csv(ws);
        
        const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
        const link = document.createElement('a');
        const fileName = `termos_referencia_${new Date().toISOString().split('T')[0]}.csv`;
        
        link.href = URL.createObjectURL(blob);
        link.download = fileName;
        link.click();
        
        this.loading = false;
        alert('Arquivo CSV exportado com sucesso!');
      } catch (error) {
        console.error('Erro ao exportar:', error);
        this.loading = false;
        alert('Erro ao exportar arquivo.');
      }
    }, 1000);
  }

  /**
   * Seleciona uma linha da tabela
   */
  selectRow(index: number): void {
    this.selectedIndex = index;
  }

  /**
   * Inicia a edição de um item
   */
  editarItem(index: number): void {
    this.itemEdicao = { ...this.termosReferencia[index] };
    this.selectedIndex = index;
    this.editingItem = true;
  }

  /**
   * Remove um item da tabela
   */
  removerItem(index: number): void {
    if (confirm('Tem certeza que deseja remover este item?')) {
      this.termosReferencia.splice(index, 1);
      this.totalItems = this.termosReferencia.length;
      this.selectedIndex = -1;
      this.atualizarArquivoAtual();
      this.salvarDadosLocais();
    }
  }

  /**
   * Salva as edições do item
   */
  salvarEdicao(): void {
    if (this.validarItem(this.itemEdicao)) {
      this.termosReferencia[this.selectedIndex] = { ...this.itemEdicao };
      this.editingItem = false;
      this.atualizarArquivoAtual();
      this.salvarDadosLocais();
    }
  }

  /**
   * Cancela a edição
   */
  cancelarEdicao(): void {
    this.editingItem = false;
    this.itemEdicao = {
      item: '',
      catalogo: '',
      descricao: '',
      descricao_detalhada: ''
    };
  }

  /**
   * Atualiza dados do arquivo atual no histórico
   */
  private atualizarArquivoAtual(): void {
    if (this.arquivoAtualId) {
      const arquivo = this.arquivosImportados.find(a => a.id === this.arquivoAtualId);
      if (arquivo) {
        arquivo.dados = [...this.termosReferencia];
        arquivo.quantidadeItens = this.termosReferencia.length;
        this.salvarHistoricoArquivos();
      }
    }
  }

  /**
   * Valida os dados do item
   */
  private validarItem(item: TermoReferencia): boolean {
    if (!item.item.trim()) {
      alert('O campo Item é obrigatório.');
      return false;
    }
    
    if (!item.catalogo.trim()) {
      alert('O campo Catálogo é obrigatório.');
      return false;
    }
    
    if (!item.descricao.trim()) {
      alert('O campo Descrição é obrigatório.');
      return false;
    }
    
    return true;
  }

  /**
   * Gera um ID único
   */
  private gerarId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
  }

  /**
   * Salva os dados no localStorage
   */
  private salvarDadosLocais(): void {
    try {
      localStorage.setItem('termosReferencia', JSON.stringify(this.termosReferencia));
      localStorage.setItem('arquivoAtualId', this.arquivoAtualId);
    } catch (error) {
      console.error('Erro ao salvar dados locais:', error);
    }
  }

  /**
   * Carrega os dados do localStorage
   */
  private carregarDadosLocais(): void {
    try {
      const dados = localStorage.getItem('termosReferencia');
      const arquivoId = localStorage.getItem('arquivoAtualId');
      
      if (dados) {
        this.termosReferencia = JSON.parse(dados);
        this.totalItems = this.termosReferencia.length;
      }
      
      if (arquivoId) {
        this.arquivoAtualId = arquivoId;
      }
    } catch (error) {
      console.error('Erro ao carregar dados locais:', error);
    }
  }

  /**
   * Salva histórico de arquivos
   */
  private salvarHistoricoArquivos(): void {
    try {
      localStorage.setItem('arquivosImportados', JSON.stringify(this.arquivosImportados));
    } catch (error) {
      console.error('Erro ao salvar histórico:', error);
    }
  }

  /**
   * Carrega histórico de arquivos
   */
  private carregarHistoricoArquivos(): void {
    try {
      const historico = localStorage.getItem('arquivosImportados');
      if (historico) {
        const arquivos = JSON.parse(historico);
        // Converter strings de data de volta para objetos Date
        this.arquivosImportados = arquivos.map((arquivo: any) => ({
          ...arquivo,
          dataImportacao: new Date(arquivo.dataImportacao)
        }));
      }
    } catch (error) {
      console.error('Erro ao carregar histórico:', error);
    }
  }

  /**
   * Obtém nome do arquivo atual
   */
  getNomeArquivoAtual(): string {
    if (this.arquivoAtualId) {
      const arquivo = this.arquivosImportados.find(a => a.id === this.arquivoAtualId);
      return arquivo ? arquivo.nome : 'Arquivo desconhecido';
    }
    return '';
  }

  /**
   * Formata data para exibição
   */
  formatarData(data: Date): string {
    return new Date(data).toLocaleString('pt-BR');
  }
}
