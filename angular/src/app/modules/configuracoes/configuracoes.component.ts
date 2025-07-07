import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
    selector: 'app-configuracoes',
    standalone: true,
    imports: [CommonModule],
    template: `
    <div class="page-container">
      <div class="page-header">
        <h1>⚙️ Configurações</h1>
        <p>Configurações do sistema e preferências</p>
      </div>
      
      <div class="page-content">
        <div class="card">
          <h3>Configurações Gerais</h3>
          <ul>
            <li>Pastas padrão do sistema</li>
            <li>Formato de exportação</li>
            <li>Idioma da interface</li>
            <li>Tema da aplicação</li>
          </ul>
        </div>
        
        <div class="card">
          <h3>Processamento</h3>
          <ul>
            <li>Configuração de OCR</li>
            <li>Padrões de extração</li>
            <li>Validação de dados</li>
            <li>Logs do sistema</li>
          </ul>
        </div>
        
        <div class="card">
          <h3>Backup e Segurança</h3>
          <ul>
            <li>Backup automático</li>
            <li>Criptografia de dados</li>
            <li>Política de retenção</li>
            <li>Auditoria de acesso</li>
          </ul>
        </div>
      </div>
    </div>
  `,
    styles: [`
    .page-container {
      padding: 2rem;
      max-width: 1200px;
      margin: 0 auto;
    }
    
    .page-header {
      margin-bottom: 2rem;
      text-align: center;
    }
    
    .page-header h1 {
      color: #2c3e50;
      margin-bottom: 0.5rem;
    }
    
    .page-header p {
      color: #7f8c8d;
      font-size: 1.1rem;
    }
    
    .page-content {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: 2rem;
    }
    
    .card {
      background: white;
      border-radius: 8px;
      padding: 2rem;
      box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
      border: 1px solid #e0e0e0;
    }
    
    .card h3 {
      color: #2c3e50;
      margin-bottom: 1rem;
    }
    
    .card ul {
      list-style: none;
      padding: 0;
    }
    
    .card li {
      padding: 0.5rem 0;
      color: #555;
      position: relative;
      padding-left: 1.5rem;
    }
    
    .card li::before {
      content: "🔧";
      position: absolute;
      left: 0;
      font-size: 0.8rem;
    }
  `]
})
export class ConfiguracoesComponent {

}
