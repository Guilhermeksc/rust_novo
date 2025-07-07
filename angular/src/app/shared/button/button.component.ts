import { Component, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';

export type ButtonVariant = 'primary' | 'secondary' | 'accent' | 'danger' | 'default';
export type ButtonSize = 'sm' | 'md' | 'lg';

@Component({
  selector: 'app-button',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './button.component.html',
  styleUrl: './button.component.css'
})
export class ButtonComponent {
  @Input() variant: ButtonVariant = 'default';
  @Input() size: ButtonSize = 'md';
  @Input() icon: string = '';
  @Input() label: string = '';
  @Input() tooltip: string = '';
  @Input() disabled: boolean = false;
  @Input() loading: boolean = false;
  @Input() fullWidth: boolean = false;
  @Input() iconOnly: boolean = false;
  
  @Output() buttonClick = new EventEmitter<void>();

  onClick(): void {
    if (!this.disabled && !this.loading) {
      this.buttonClick.emit();
    }
  }

  getButtonClasses(): string {
    const classes = ['btn'];
    
    // Variant classes
    classes.push(`btn-${this.variant}`);
    
    // Size classes
    classes.push(`btn-${this.size}`);
    
    // State classes
    if (this.disabled) classes.push('btn-disabled');
    if (this.loading) classes.push('btn-loading');
    if (this.fullWidth) classes.push('btn-full-width');
    if (this.iconOnly) classes.push('btn-icon-only');
    
    return classes.join(' ');
  }

  // Verifica se o ícone é uma imagem (arquivo)
  isImageIcon(): boolean {
    if (!this.icon || typeof this.icon !== 'string') return false;
    
    return this.icon.includes('.png') ||
           this.icon.includes('.jpg') ||
           this.icon.includes('.jpeg') ||
           this.icon.includes('.gif') ||
           this.icon.includes('.svg') ||
           this.icon.includes('.webp');
  }

  // Verifica se o ícone é um emoji ou texto
  isEmojiIcon(): boolean {
    return !!this.icon && !this.isImageIcon();
  }
}
