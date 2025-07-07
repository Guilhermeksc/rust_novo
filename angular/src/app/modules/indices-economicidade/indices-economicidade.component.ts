import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
    selector: 'app-indices-economicidade',
    standalone: true,
    imports: [CommonModule],
    template: `
    <div class="page-container">
      <div class="page-header">
        <h1>📊 Índices de Economicidade</h1>
        <p>Análise e cálculo de economicidade em processos licitatórios</p>
      </div>
      
      <div class="page-content">
        <div class="card">
          <h3>Indicadores Calculados</h3>
          <ul>
            <li>Economia obtida por processo</li>
            <li>Percentual de redução</li>
            <li>Comparativo com preço de referência</li>
            <li>Análise de competitividade</li>
          </ul>
        </div>
        
        <div class="card">
          <h3>Relatórios Gerados</h3>
          <ul>
            <li>Relatório de economicidade</li>
            <li>Gráficos comparativos</li>
            <li>Histórico de economias</li>
            <li>Análise de tendências</li>
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
      content: "📈";
      position: absolute;
      left: 0;
      font-size: 0.8rem;
    }
  `]
})
export class IndicesEconomicidadeComponent {

}
