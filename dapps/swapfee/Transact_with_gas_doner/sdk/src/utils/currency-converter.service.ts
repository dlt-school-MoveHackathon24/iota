import { Injectable } from '@nestjs/common';

@Injectable()
export class CurrencyConverterService {
  // Conversione da IOTA a MINT (1 IOTA = 100 MINT)
  convertIotaToMint(iotaAmount: number): number {
    return iotaAmount * 100;
  }

  // Conversione da MINT a IOTA (1 MINT = 0.01 IOTA)
  convertMintToIota(mintAmount: number): number {
    return mintAmount / 100;
  }
}
