import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SelecionarPastaComponent } from './selecionar-pasta.component';

describe('SelecionarPastaComponent', () => {
  let component: SelecionarPastaComponent;
  let fixture: ComponentFixture<SelecionarPastaComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [SelecionarPastaComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SelecionarPastaComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
