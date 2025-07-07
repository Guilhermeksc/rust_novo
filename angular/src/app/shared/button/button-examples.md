# Button Component - Exemplos de Uso

O componente `app-button` é um botão reutilizável e padronizado para toda a aplicação.

## Propriedades

| Propriedade | Tipo | Padrão | Descrição |
|-------------|------|--------|-----------|
| `variant` | `'primary' \| 'secondary' \| 'accent' \| 'danger' \| 'default'` | `'default'` | Estilo visual do botão |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | Tamanho do botão |
| `icon` | `string` | `''` | Ícone emoji a ser exibido |
| `label` | `string` | `''` | Texto do botão |
| `tooltip` | `string` | `''` | Texto do tooltip |
| `disabled` | `boolean` | `false` | Se o botão está desabilitado |
| `loading` | `boolean` | `false` | Se o botão está em estado de carregamento |
| `fullWidth` | `boolean` | `false` | Se o botão ocupa toda a largura disponível |
| `iconOnly` | `boolean` | `false` | Se o botão mostra apenas o ícone |

## Eventos

| Evento | Descrição |
|--------|-----------|
| `buttonClick` | Emitido quando o botão é clicado |

## Exemplos de Uso

### Botão Básico
```html
<app-button 
  label="Clique aqui" 
  (buttonClick)="onButtonClick()">
</app-button>
```

### Botão com Ícone
```html
<app-button 
  variant="primary"
  icon="💾"
  label="Salvar"
  (buttonClick)="onSave()">
</app-button>
```

### Botão com Ícone de Imagem
```html
<app-button 
  variant="primary"
  icon="assets/icons/excel.png"
  label="Abrir Excel"
  (buttonClick)="onOpenExcel()">
</app-button>
```

### Botão Apenas com Ícone
```html
<app-button 
  variant="secondary"
  icon="🔄"
  iconOnly="true"
  tooltip="Atualizar"
  (buttonClick)="onRefresh()">
</app-button>
```

### Botão Apenas com Ícone de Imagem
```html
<app-button 
  variant="secondary"
  icon="assets/icons/excel.png"
  iconOnly="true"
  tooltip="Abrir Excel"
  (buttonClick)="onOpenExcel()">
</app-button>
```

### Botão em Estado de Carregamento
```html
<app-button 
  variant="primary"
  label="Salvando..."
  loading="true"
  (buttonClick)="onSave()">
</app-button>
```

### Botão Desabilitado
```html
<app-button 
  variant="danger"
  icon="🗑️"
  label="Excluir"
  disabled="true"
  (buttonClick)="onDelete()">
</app-button>
```

### Botão de Largura Completa
```html
<app-button 
  variant="primary"
  label="Enviar Formulário"
  fullWidth="true"
  (buttonClick)="onSubmit()">
</app-button>
```

### Diferentes Tamanhos
```html
<!-- Pequeno -->
<app-button 
  size="sm"
  variant="secondary"
  label="Pequeno"
  (buttonClick)="onClick()">
</app-button>

<!-- Médio (padrão) -->
<app-button 
  size="md"
  variant="primary"
  label="Médio"
  (buttonClick)="onClick()">
</app-button>

<!-- Grande -->
<app-button 
  size="lg"
  variant="accent"
  label="Grande"
  (buttonClick)="onClick()">
</app-button>
```

### Diferentes Variantes
```html
<!-- Padrão -->
<app-button 
  variant="default"
  label="Padrão"
  (buttonClick)="onClick()">
</app-button>

<!-- Primário -->
<app-button 
  variant="primary"
  label="Primário"
  (buttonClick)="onClick()">
</app-button>

<!-- Secundário -->
<app-button 
  variant="secondary"
  label="Secundário"
  (buttonClick)="onClick()">
</app-button>

<!-- Accent -->
<app-button 
  variant="accent"
  label="Accent"
  (buttonClick)="onClick()">
</app-button>

<!-- Perigo -->
<app-button 
  variant="danger"
  label="Perigo"
  (buttonClick)="onClick()">
</app-button>
```

## Tipos de Ícones Suportados

### Emojis
O componente suporta emojis diretamente:
```html
<app-button icon="💾" label="Salvar"></app-button>
<app-button icon="🔄" label="Atualizar"></app-button>
<app-button icon="🗑️" label="Excluir"></app-button>
```

### Imagens
O componente detecta automaticamente quando o ícone é uma imagem:
```html
<app-button icon="assets/icons/excel.png" label="Excel"></app-button>
<app-button icon="assets/icons/pdf.png" label="PDF"></app-button>
<app-button icon="assets/icons/word.png" label="Word"></app-button>
```

**Formatos suportados**: PNG, JPG, JPEG, GIF, SVG, WebP

**Estilização automática**: As imagens são automaticamente estilizadas para:
- Manter proporções corretas
- Ajustar tamanho baseado no tamanho do botão
- Aplicar filtro branco para contrastar com o fundo

## Cores das Variantes

- **default**: Transparente com borda branca
- **primary**: Verde (sucesso, ações principais)
- **secondary**: Azul (ações secundárias)
- **accent**: Roxo (destaque especial)
- **danger**: Vermelho (ações perigosas, exclusões)

## Responsividade

Em telas menores (mobile):
- Labels são ocultadas automaticamente
- Botões mostram apenas ícones
- Tamanhos são reduzidos proporcionalmente

## Acessibilidade

- Suporte completo a navegação por teclado
- Estados de foco visíveis
- Tooltips para contexto adicional
- Indicadores de carregamento acessíveis 