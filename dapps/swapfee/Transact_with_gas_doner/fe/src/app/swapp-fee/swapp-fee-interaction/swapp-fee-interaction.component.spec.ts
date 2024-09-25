import { ComponentFixture, TestBed } from '@angular/core/testing';

import { SwappFeeInteractionComponent } from './swapp-fee-interaction.component';

describe('SwappFeeInteractionComponent', () => {
  let component: SwappFeeInteractionComponent;
  let fixture: ComponentFixture<SwappFeeInteractionComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [SwappFeeInteractionComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(SwappFeeInteractionComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
