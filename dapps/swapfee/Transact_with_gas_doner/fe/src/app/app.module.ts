import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';
import { SwappFeeModule } from './swapp-fee/swapp-fee.module'; // Importa il nuovo modulo
import { HttpClientModule, HTTP_INTERCEPTORS } from '@angular/common/http';
import { FormsModule } from '@angular/forms'; // Importa FormsModule

@NgModule({
  declarations: [
    AppComponent,
  ],
  imports: [
    BrowserModule,
    AppRoutingModule,
    SwappFeeModule, // Aggiungi SwappFeeModule qui
    FormsModule,
    HttpClientModule // Usa HttpClientModule per la compatibilità
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
