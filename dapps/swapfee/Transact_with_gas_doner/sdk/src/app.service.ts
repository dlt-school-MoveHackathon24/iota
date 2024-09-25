import { Injectable } from '@nestjs/common';

@Injectable()
export class AppService {
  // Funzione per elaborare i dati del form
  processFormData(formData: any): any {
    // Estrai i campi dal body
    const { senderAddress, destinationAddress, selectedCurrency, maxGasAmount } = formData;

    // Esegui ulteriori validazioni o calcoli qui, se necessario
    const processedData = {
      senderAddress: senderAddress || 'N/A',
      destinationAddress: destinationAddress || 'N/A',
      currency: selectedCurrency || 'Unknown',
      maxGasAmount: maxGasAmount ? parseFloat(maxGasAmount) : 0, // Convertire maxGasAmount in numero
      timestamp: new Date(), // Aggiungi un timestamp
    };

    return processedData; // Restituisci i dati elaborati
  }
}
