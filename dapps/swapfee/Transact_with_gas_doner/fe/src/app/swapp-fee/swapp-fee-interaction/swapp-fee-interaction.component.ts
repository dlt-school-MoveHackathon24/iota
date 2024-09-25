import { Component } from '@angular/core';
import { HttpClient } from '@angular/common/http';  
import { catchError } from 'rxjs/operators';
import { throwError } from 'rxjs';

@Component({
  selector: 'app-swapp-fee-interaction',
  templateUrl: './swapp-fee-interaction.component.html',
  styleUrls: ['./swapp-fee-interaction.component.css']
})
export class SwappFeeInteractionComponent {
  senderAddress: string = ''; // Indirizzo del mittente
  sponsorAddress: string = ''; // Indirizzo dello sponsor
  destinationAddress: string = ''; // Indirizzo di destinazione
  selectedCurrency: string = 'MINT'; // Valuta predefinita
 // maxGasAmount: string = ''; // Quantità massima di gas in MINT
  idObjectToTransfer: string = ''; // ID dell'oggetto da trasferire
  currencies: string[] = ['MINT']; // Array con un'unica valuta
  sponsorIOTAObject: string = ''; //indirizzo oggetto IOTA dello sponsor

  transactionSuccess: boolean = false; // Stato della transazione
  errorMessage: string = ''; // Messaggio di errore
  isSubmitting: boolean = false; // Stato per gestire l'invio del form

  constructor(private http: HttpClient) {}

  onSubmit() {
    if (this.isSubmitting) return; // Evita l'invio multiplo durante la richiesta

    this.isSubmitting = true; // Imposta lo stato a "in invio"

    const formData = {
      senderAddress: this.senderAddress,
      sponsorAddress: this.sponsorAddress,
      sponsorIOTAObject: this.sponsorIOTAObject,
      destinationAddress: this.destinationAddress,
      selectedCurrency: this.selectedCurrency,
   //   maxGasAmount: this.maxGasAmount,
      idObjectToTransfer: this.idObjectToTransfer
    };

    this.http.post('http://localhost:3000/api/submit-fee', formData)
      .pipe(
        catchError(error => {
          console.error('Errore durante l\'invio del form:', error);
          this.errorMessage = 'Errore durante la transazione: ' + (error.error?.message || 'Si è verificato un problema');
          this.transactionSuccess = false; // Nasconde il messaggio di successo in caso di errore
          this.isSubmitting = false; // Rende possibile un nuovo submit
          return throwError(error);
        })
      )
      .subscribe((response: any) => {
        if (response.success) {
          this.transactionSuccess = true; // Mostra il messaggio di successo
          this.errorMessage = ''; // Resetta eventuali messaggi di errore
        } else {
          this.transactionSuccess = false; // Nasconde il messaggio di successo
          this.errorMessage = response.message || 'Transazione fallita.'; // Mostra l'errore
        }
        this.isSubmitting = false; // Permette di inviare di nuovo il form
      });
  }
}
