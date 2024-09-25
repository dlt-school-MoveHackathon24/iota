// swapp-fee.module.ts
import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { SwappFeeInteractionComponent } from './swapp-fee-interaction/swapp-fee-interaction.component';
import { FormsModule } from '@angular/forms';  // Per il binding dei form
import { HttpClientModule } from '@angular/common/http';  // Per le chiamate HTTP

@NgModule({
  declarations: [
    SwappFeeInteractionComponent
  ],
  imports: [
    CommonModule,
    FormsModule,         // Necessario per il two-way binding con [(ngModel)]
    HttpClientModule     // Necessario per fare richieste HTTP
  ],
  exports: [
    SwappFeeInteractionComponent  // Esporta il componente in modo che sia accessibile da AppModule
  ]
})
export class SwappFeeModule { }
