import { Controller, Get, Post, Body, Query, NotFoundException, BadRequestException } from '@nestjs/common';
import { AppService } from './app.service';
import { CurrencyConverterService } from './utils/currency-converter.service';
import { AccountService } from './utils/account.service';
import { FixedAccountService } from './utils/fixed-account.service';
import { FixedAccount } from './utils/fixed-account.model';
import { IotaObjectService } from './utils/iota-object.service'; // Importa il servizio degli oggetti IOTA
import { exec } from 'child_process';


@Controller('api') // Definisce il percorso base per tutti i metodi del controller
export class AppController {
  constructor(
    private readonly appService: AppService,
    private readonly currencyConverterService: CurrencyConverterService,
    private readonly accountService: AccountService,
    private readonly fixedAccountService: FixedAccountService, // Aggiungi il servizio degli account fissi
    private readonly iotaObjectService: IotaObjectService // Inietta il servizio degli oggetti IOTA
  ) {}

  @Get('test') // Endpoint GET per verificare che il server stia rispondendo
  getTest(): string {
    return 'Server is working!';
  }


// Post submit-fee funzionante

 /* @Post('submit-fee')
  submitFee(@Body() formData: any) {
    console.log('Dati ricevuti:', formData);
  
    const senderAddress = formData.senderAddress; // Assicurati che ci sia un campo per l'indirizzo del mittente
    const mintAmount = Number(formData.maxGasAmount); // Converti in numero
  
    // Recupera gli account fissi
    const fixedAccounts: FixedAccount[] = this.fixedAccountService.getFixedAccounts();
    
    // Trova l'account corrispondente all'indirizzo del mittente
    const senderAccount = fixedAccounts.find(account => account.address === senderAddress);
    
    // Controllo se l'account esiste
    if (!senderAccount) {
      return {
        success: false,
        message: 'L\'indirizzo del mittente non esiste nei fixed accounts.'
      };
    }
  
    // Controlla se il maxGasAmount è minore o uguale al mintBalance
    if (mintAmount > senderAccount.mintBalance) {
      return {
        success: false,
        message: 'Il maxGasAmount specificato è maggiore del balance disponibile nell\'account.'
      };
    }
  
    // Converti il maxGasAmount da MINT a IOTA
    const iotaAmount = this.currencyConverterService.convertMintToIota(mintAmount);
  
    return {
      success: true,
      message: 'Dati ricevuti con successo',
      maxGasAmount: mintAmount,  // MINT
      iotaAmount: iotaAmount,     // IOTA
    };
  }
*/

//tentativo per aprire bash
@Post('submit-fee')
submitFee(@Body() formData: any) {
  console.log('Dati ricevuti:', formData);

  const senderAddress = formData.senderAddress; // Indirizzo del mittente
  const destinationAddress = formData.destinationAddress; // Indirizzo di destinazione
  const sponsorAddress = formData.sponsorAddress; // Indirizzo dello sponsor
  const GasAmountInMint = 0; // Quantità di gas in MINT
  const idObjectToTransfer = formData.idObjectToTransfer; // ID dell'oggetto da trasferire
  const sponsorIOTAObject = formData.sponsorIOTAObject;
  // Recupera gli account fissi
  const fixedAccounts: FixedAccount[] = this.fixedAccountService.getFixedAccounts();

  // Trova l'account corrispondente all'indirizzo del mittente
  const senderAccount = fixedAccounts.find(account => account.address === senderAddress);

  // Controllo se l'account del mittente esiste
  if (!senderAccount) {
    return {
      success: false,
      message: 'L\'indirizzo del mittente non esiste nei fixed accounts.'
    };
  }

  // Controlla se il GasAmountInMint è minore o uguale al saldo disponibile nell'account del mittente
 /* if (GasAmountInMint > senderAccount.mintBalance) {
    return {
      success: false,
      message: 'Il maxGasAmount specificato è maggiore del balance disponibile nell\'account.'
    };
  }*/
 

  // Converti il maxGasAmount da MINT a IOTA
  const iotaAmount = this.currencyConverterService.convertMintToIota(GasAmountInMint);

  // Recupera il primo oggetto IOTA associato allo sponsor
  /*const iotaObjects = this.iotaObjectService.getIotaObjectsForAccount(sponsorAddress);
  if (!iotaObjects.length) {
    return {
      success: false,
      message: 'Nessun oggetto IOTA trovato per l\'indirizzo dello sponsor.'
    };
  }
  const firstIotaObject = iotaObjects[0]; // Recupera il primo oggetto IOTA
*/
  // Fase 1: Esegui lo switch sull'account sponsor
  const switchToSponsorCommand = `iota client switch --address ${sponsorAddress}`;
  exec(switchToSponsorCommand, (error, stdout, stderr) => {
    if (error) {
      console.error(`Errore durante lo switch sull'account sponsor: ${error.message}`);
      return {
        success: false,
        message: `Errore durante lo switch sull'account sponsor: ${error.message}`,
        output: error.message
      };
    }

    if (stderr) {
      console.error(`stderr durante lo switch sull'account sponsor: ${stderr}`);
      return {
        success: false,
        message: `Errore durante lo switch sull'account sponsor: ${stderr}`,
        output: stderr
      };
    }

    console.log(`Risultato dello switch sull'account sponsor: ${stdout}`);

    // Fase 2: Trasferisci IOTA dallo sponsor al mittente (sender)
    const transferToSenderCommand = `iota client transfer --to ${senderAddress} --object-id ${sponsorIOTAObject} --gas-budget 5000000`;
    exec(transferToSenderCommand, (error, stdout, stderr) => {
      if (error) {
        console.error(`Errore durante il trasferimento di IOTA dallo sponsor al sender: ${error.message}`);
        return {
          success: false,
          message: `Errore durante il trasferimento di IOTA dallo sponsor al sender: ${error.message}`,
          output: error.message
        };
      }

      if (stderr) {
        console.error(`stderr durante il trasferimento di IOTA: ${stderr}`);
        return {
          success: false,
          message: `Errore durante il trasferimento di IOTA: ${stderr}`,
          output: stderr
        };
      }

      console.log(`Risultato del trasferimento di IOTA: ${stdout}`);

      // Fase 3: Esegui lo switch sull'account del sender
      const switchToSenderCommand = `iota client switch --address ${senderAddress}`;
      exec(switchToSenderCommand, (error, stdout, stderr) => {
        if (error) {
          console.error(`Errore durante lo switch sull'account sender: ${error.message}`);
          return {
            success: false,
            message: `Errore durante lo switch sull'account sender: ${error.message}`,
            output: error.message
          };
        }

        if (stderr) {
          console.error(`stderr durante lo switch sull'account sender: ${stderr}`);
          return {
            success: false,
            message: `Errore durante lo switch sull'account sender: ${stderr}`,
            output: stderr
          };
        }

        console.log(`Risultato dello switch sull'account sender: ${stdout}`);

        // Fase 4: Trasferisci i MINT al destinatario
        const transferCommand = `iota client transfer --to ${destinationAddress} --object-id ${idObjectToTransfer} --gas-budget 5000000`;
        exec(transferCommand, (error, stdout, stderr) => {
          if (error) {
            console.error(`Errore durante il trasferimento finale: ${error.message}`);
            return {
              success: false,
              message: `Errore durante il trasferimento finale: ${error.message}`,
              output: error.message
            };
          }

          if (stderr) {
            console.error(`stderr durante il trasferimento finale: ${stderr}`);
            return {
              success: false,
              message: `Errore durante il trasferimento finale: ${stderr}`,
              output: stderr
            };
          }

          console.log(`Risultato del trasferimento finale: ${stdout}`);

          // Risposta finale con successo
          return {
            success: true,
            message: 'Tutti i trasferimenti eseguiti con successo!',
            maxGasAmount: GasAmountInMint,  // MINT
            iotaAmount: iotaAmount,    // IOTA
            transferredObjectId: idObjectToTransfer, // L'ID dell'oggetto trasferito
            output: stdout
          };
        });
      });
    });
  });

  // Ritorna un messaggio di successo immediato (prima che il comando finisca)
  return {
    success: true,
    message: 'Operazioni in corso, attendere il completamento dei comandi.',
    maxGasAmount: GasAmountInMint,  // MINT
    iotaAmount: iotaAmount,    // IOTA
    transferredObjectId: idObjectToTransfer
  };
}






  // Endpoint POST per aggiungere un account
  @Post('add-account')
  addAccount(
    @Body('alias') alias: string,
    @Body('address') address: string,
    @Body('balance') balance: number,
  ) {
    console.log(`Aggiungendo account: Alias: ${alias}, Address: ${address}, Balance: ${balance}`);
    return this.accountService.addAccount(alias, address, balance);
  }

  // Endpoint GET per ottenere tutti gli account
  @Get('accounts')
  getAccounts() {
    return this.accountService.getAccounts();
  }

  // Endpoint GET per ottenere un account tramite alias
  @Get('account')
  getAccountByAlias(@Query('alias') alias: string) {
    return this.accountService.getAccountByAlias(alias);
  }

  // Endpoint GET per ottenere gli account fissi
  @Get('fixed-accounts')
  getFixedAccounts() {
    return this.fixedAccountService.getFixedAccounts(); // Usa il servizio degli account fissi
  }
}
