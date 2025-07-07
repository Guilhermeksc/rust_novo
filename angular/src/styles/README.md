# 🎨 Sistema de Tipografia Moderna

Este sistema oferece uma abordagem moderna e consistente para tipografia em toda a aplicação Licitação 360.

## 🚀 Funcionalidades

### ✨ Fontes Modernas
- **Inter**: Fonte principal com excelente legibilidade
- **JetBrains Mono**: Fonte monoespaçada para código
- Fallbacks system fonts para melhor performance

### 🎯 Variáveis CSS
- Sistema de design consistente
- Fácil manutenção e personalização
- Escalabilidade responsiva

### 🔧 Classes Utilitárias
- Inspiradas no Tailwind CSS
- Rápida prototipagem
- Desenvolvimento eficiente

## 📁 Estrutura dos Arquivos

```
src/styles/
├── typography.css      # Variáveis CSS e estilos base
├── utilities.css       # Classes utilitárias
└── README.md          # Esta documentação
```

## 🎨 Variáveis CSS Disponíveis

### 📝 Fontes
```css
--font-primary: 'Inter', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', monospace;
--font-system: system-ui, sans-serif;
```

### ⚖️ Pesos de Fonte
```css
--fw-light: 300;
--fw-regular: 400;
--fw-medium: 500;
--fw-semibold: 600;
--fw-bold: 700;
```

### 📏 Tamanhos de Fonte
```css
--fs-xs: 0.75rem;     /* 12px */
--fs-sm: 0.875rem;    /* 14px */
--fs-base: 1rem;      /* 16px */
--fs-lg: 1.125rem;    /* 18px */
--fs-xl: 1.25rem;     /* 20px */
--fs-2xl: 1.5rem;     /* 24px */
--fs-3xl: 1.875rem;   /* 30px */
--fs-4xl: 2.25rem;    /* 36px */
--fs-5xl: 3rem;       /* 48px */
--fs-6xl: 3.75rem;    /* 60px */
```

### 📐 Alturas de Linha
```css
--lh-none: 1;
--lh-tight: 1.25;
--lh-snug: 1.375;
--lh-normal: 1.5;
--lh-relaxed: 1.625;
--lh-loose: 2;
```

### 🎯 Cores de Texto
```css
--text-primary: #1a202c;
--text-secondary: #4a5568;
--text-muted: #718096;
--text-accent: #3182ce;
--text-success: #38a169;
--text-warning: #d69e2e;
--text-error: #e53e3e;
--text-inverse: #ffffff;
```

### 📏 Espaçamentos
```css
--space-1: 0.25rem;   /* 4px */
--space-2: 0.5rem;    /* 8px */
--space-3: 0.75rem;   /* 12px */
--space-4: 1rem;      /* 16px */
--space-5: 1.25rem;   /* 20px */
--space-6: 1.5rem;    /* 24px */
--space-8: 2rem;      /* 32px */
--space-10: 2.5rem;   /* 40px */
--space-12: 3rem;     /* 48px */
```

## 🔧 Classes Utilitárias

### 📝 Tamanhos de Fonte
```css
.text-xs     /* 12px */
.text-sm     /* 14px */
.text-base   /* 16px */
.text-lg     /* 18px */
.text-xl     /* 20px */
.text-2xl    /* 24px */
.text-3xl    /* 30px */
.text-4xl    /* 36px */
.text-5xl    /* 48px */
.text-6xl    /* 60px */
```

### ⚖️ Pesos de Fonte
```css
.font-light     /* 300 */
.font-normal    /* 400 */
.font-medium    /* 500 */
.font-semibold  /* 600 */
.font-bold      /* 700 */
```

### 🎨 Cores de Texto
```css
.text-primary     /* Cor principal */
.text-secondary   /* Cor secundária */
.text-muted       /* Cor suavizada */
.text-accent      /* Cor de destaque */
.text-success     /* Verde */
.text-warning     /* Amarelo */
.text-error       /* Vermelho */
.text-inverse     /* Branco */
```

### 📐 Alturas de Linha
```css
.leading-none      /* 1 */
.leading-tight     /* 1.25 */
.leading-snug      /* 1.375 */
.leading-normal    /* 1.5 */
.leading-relaxed   /* 1.625 */
.leading-loose     /* 2 */
```

### 🎯 Espaçamento entre Letras
```css
.tracking-tighter  /* -0.05em */
.tracking-tight    /* -0.025em */
.tracking-normal   /* 0em */
.tracking-wide     /* 0.025em */
.tracking-wider    /* 0.05em */
.tracking-widest   /* 0.1em */
```

## 🔘 Componentes Modernos

### Botões
```html
<button class="btn-modern btn-primary">
  🚀 Botão Principal
</button>

<button class="btn-modern btn-secondary">
  📝 Botão Secundário
</button>

<button class="btn-modern btn-accent">
  ⭐ Botão Destaque
</button>
```

### Cards
```html
<div class="card-modern">
  <h3 class="text-lg font-semibold text-primary mb-2">
    Título do Card
  </h3>
  <p class="text-secondary">
    Conteúdo do card...
  </p>
</div>

<div class="card-modern card-hover">
  <!-- Card com efeito hover -->
</div>
```

### Formulários
```html
<div>
  <label class="label-modern">Label</label>
  <input type="text" class="input-modern" placeholder="Placeholder">
</div>
```

## 💡 Exemplos de Uso

### Título Principal
```html
<h1 class="text-4xl font-bold text-primary mb-6 leading-tight tracking-tight">
  Título Principal
</h1>
```

### Parágrafo Comum
```html
<p class="text-base text-secondary leading-relaxed mb-4">
  Este é um parágrafo com espaçamento relaxado e cor secundária.
</p>
```

### Texto de Destaque
```html
<span class="text-lg font-semibold text-accent">
  Texto importante em destaque
</span>
```

### Card com Conteúdo
```html
<div class="card-modern mb-6">
  <h2 class="text-2xl font-semibold text-primary mb-4 tracking-tight">
    Título da Seção
  </h2>
  <p class="text-secondary leading-relaxed">
    Conteúdo da seção...
  </p>
</div>
```

## 📱 Responsividade

O sistema é totalmente responsivo com breakpoints:

- **Mobile**: < 768px
- **Tablet**: 768px - 1024px
- **Desktop**: > 1024px

Os tamanhos de fonte são ajustados automaticamente para cada breakpoint.

## 🎨 Personalização

### Alterando Cores
```css
:root {
  --text-primary: #seu-valor;
  --text-accent: #seu-valor;
}
```

### Adicionando Novos Tamanhos
```css
:root {
  --fs-7xl: 4rem;
}

.text-7xl {
  font-size: var(--fs-7xl);
}
```

### Criando Novos Componentes
```css
.meu-componente {
  font-family: var(--font-primary);
  font-size: var(--fs-base);
  font-weight: var(--fw-medium);
  color: var(--text-primary);
  padding: var(--space-4);
}
```

## 🚀 Vantagens

1. **Consistência**: Design system unificado
2. **Manutenibilidade**: Fácil de atualizar
3. **Performance**: Fontes otimizadas
4. **Acessibilidade**: Contraste e legibilidade
5. **Produtividade**: Desenvolvimento mais rápido

## 📚 Recursos Adicionais

- [Inter Font](https://rsms.me/inter/)
- [JetBrains Mono](https://www.jetbrains.com/lp/mono/)
- [CSS Custom Properties](https://developer.mozilla.org/pt-BR/docs/Web/CSS/Using_CSS_custom_properties)
- [Design System Guidelines](https://designsystemsrepo.com/)

## 🔄 Migração de Estilos Antigos

Para migrar estilos existentes:

1. Substitua `font-family` por variáveis CSS
2. Use classes utilitárias para tamanhos e pesos
3. Aplique o sistema de cores consistente
4. Utilize espaçamentos padronizados

### Exemplo de Migração

**Antes:**
```css
.titulo {
  font-family: Arial, sans-serif;
  font-size: 24px;
  font-weight: bold;
  color: #333;
  margin-bottom: 16px;
}
```

**Depois:**
```html
<h2 class="text-2xl font-bold text-primary mb-4">
  Título
</h2>
```

---

**Desenvolvido com ❤️ para o projeto Licitação 360** 