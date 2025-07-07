import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';
import { invoke } from "@tauri-apps/api/core";
import { NavigationService } from './services/navigation.service';
import { SidebarComponent } from './modules/sidebar/sidebar.component';
import { TermoReferenciaComponent } from './modules/termo-referencia/termo-referencia.component';
import { ProcessarHomologacaoComponent } from './modules/processar-homologacao/processar-homologacao.component';
import { DadosEmpresasComponent } from './modules/dados-empresas/dados-empresas.component';
import { GerarAtaComponent } from './modules/gerar-ata/gerar-ata.component';
import { IndicesEconomicidadeComponent } from './modules/indices-economicidade/indices-economicidade.component';
import { ConfiguracoesComponent } from './modules/configuracoes/configuracoes.component';
import { HeaderContentComponent } from './shared/header-content/header-content.component';

@Component({
    selector: 'app-root',
    standalone: true,
    imports: [
        CommonModule,
        RouterOutlet,
        SidebarComponent,
        TermoReferenciaComponent,
        ProcessarHomologacaoComponent,
        DadosEmpresasComponent,
        GerarAtaComponent,
        IndicesEconomicidadeComponent,
        ConfiguracoesComponent,
        HeaderContentComponent
    ],
    templateUrl: './app.component.html',
    styleUrl: './app.component.css'
})
export class AppComponent implements OnInit {
  title = 'Licitação 360';
  currentPage: string = 'processar-homologacao';
  sidebarCollapsed: boolean = false;
  sidebarOpen: boolean = false;

  constructor(private navigationService: NavigationService) {}

  ngOnInit(): void {
    this.navigationService.currentPage$.subscribe(page => {
      this.currentPage = page;
    });

    this.navigationService.sidebarCollapsed$.subscribe(collapsed => {
      this.sidebarCollapsed = collapsed;
    });

    this.navigationService.sidebarOpen$.subscribe(open => {
      this.sidebarOpen = open;
    });
  }

  toggleMobileSidebar(): void {
    this.navigationService.toggleSidebarOpen();
  }

  async greet(): Promise<void> {
    // Greet command for testing
    const greetMessage = await invoke<string>("greet", { name: "Tauri" });
    console.log(greetMessage);
  }
}
