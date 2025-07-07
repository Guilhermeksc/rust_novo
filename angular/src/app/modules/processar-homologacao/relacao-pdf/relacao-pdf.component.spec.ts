import { ComponentFixture, TestBed } from '@angular/core/testing';

import { RelacaoPdfComponent } from './relacao-pdf.component';

describe('RelacaoPdfComponent', () => {
  let component: RelacaoPdfComponent;
  let fixture: ComponentFixture<RelacaoPdfComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [RelacaoPdfComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(RelacaoPdfComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
